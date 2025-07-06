use std::{fmt::Debug, collections::HashMap};

use chrono::{prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods};
use rand::{distributions::{Distribution, Standard}, Rng};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::config_variables::DATE_FORMAT;

use crate::schema::*;
use crate::database::connection;

use super::{Person, Team, Work, Requirement, Capability};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = roles)]
#[diesel(belongs_to(Person))]
#[diesel(belongs_to(Team))]
/// Intermediary data structure between Person and team
/// Referenced by Person
pub struct Role {
    pub id: Uuid,
    pub person_id: Option<Uuid>, // You can have an empty role on a team
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    pub active: bool,
    // HR info - this will be another module - just here for expediency
    pub military_occupation: MilitaryOccupation,
    pub rank: Rank,

    pub start_datestamp: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[Object]
impl Role {

    pub async fn id(&self) -> Uuid {
        self.id
    }

    pub async fn person(&self) -> Option<Person> {

        match self.person_id {
            Some(p) => Some(Person::get_by_id(&p).unwrap()),
            None => None
        }
    }

    pub async fn team(&self) -> Result<Team> {
        Team::get_by_id(&self.team_id)
    }

    pub async fn title_english(&self) -> Result<String> {
        Ok(self.title_en.to_owned())
    }

    pub async fn title_french(&self) -> Result<String> {
        Ok(self.title_fr.to_owned())
    }

    /// Returns the sum effort of all active work underway
    /// Maximum work should be around 10
    pub async fn effort(&self) -> Result<i32> {
        Work::sum_role_effort(&self.id)
    }

    /// Returns a vector of the work undertaken by this role
    pub async fn work(&self) -> Result<Vec<Work>> {
        Work::get_by_role_id(&self.id)
    }

    pub async fn active(&self) -> Result<String> {
        if self.active {
            Ok("Active".to_string())
        } else {
            Ok("INACTIVE".to_string())
        }
    }

    pub async fn requirements(&self) -> Result<Vec<Requirement>> {
        Requirement::get_by_role_id(self.id)
    }

    pub async fn military_occupation(&self) -> Result<String> {
        Ok(self.military_occupation.to_string())
    }

    pub async fn rank(&self) -> Result<Rank> {
        Ok(self.rank)
    }

    pub async fn start_date(&self) -> Result<String> {
        Ok(self.start_datestamp.format(DATE_FORMAT).to_string())
    }

    pub async fn end_date(&self) -> Result<String> {
        match self.end_date {
            Some(d) => Ok(d.format(DATE_FORMAT).to_string()),
            None => Ok("Still Active".to_string())
        }
    }

    pub async fn created_at(&self) -> Result<String> {
        Ok(self.created_at.format(DATE_FORMAT).to_string())
    }

    pub async fn updated_at(&self) -> Result<String> {
        Ok(self.updated_at.format(DATE_FORMAT).to_string())
    }

    pub async fn find_matches(&self) -> Result<Vec<Person>> {

        let requirements = Requirement::get_by_role_id(self.id)?;

        find_people_by_requirements_met(requirements)
    }
}


// Non Graphql
impl Role {
    pub fn create(role: &NewRole) -> Result<Role> {
        let mut conn = connection()?;

        let res = diesel::insert_into(roles::table)
        .values(role)
        .get_result(&mut conn)?;
        
        Ok(res)
    }

    pub fn batch_create(roles: Vec<NewRole>) -> Result<usize> {
        let mut conn = connection()?;

        let res = diesel::insert_into(roles::table)
            .values(roles)
            .execute(&mut conn)?;
        
        Ok(res)
    }
    
    pub fn get_or_create(role: &NewRole) -> Result<Role> {
        let mut conn = connection()?;

        let res = roles::table
        .filter(roles::person_id.eq(&role.person_id))
        .distinct()
        .first(&mut conn);
        
        let role = match res {
            Ok(p) => p,
            Err(e) => {
                // Role not found
                println!("{:?}", e);
                let p = Role::create(role).expect("Unable to create role");
                p
            }
        };
        Ok(role)
    }

    pub fn get_all_active() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table
            .filter(roles::active.eq(true))
            .load::<Role>(&mut conn)?;
        Ok(roles)
    }

    pub fn get_active(count: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table
            .filter(roles::active.eq(true))
            .limit(count)
            .load::<Role>(&mut conn)?;
        
        Ok(roles)
    }

    pub fn count() -> Result<i64> {
        let mut conn = connection()?;

        let res = roles::table
            .count()
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let role = roles::table.filter(roles::id.eq(id)).first(&mut conn)?;
        Ok(role)
    }

    pub fn get_active_vacant_by_ids(ids: &Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table
            .filter(roles::id.eq_any(ids))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_null())
            .load::<Self>(&mut conn)?;
        Ok(roles)
    }

    pub fn get_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Returns active and occupied roles by a team_id
    pub fn get_occupied_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_not_null())
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Returns vacant and active roles
    pub fn get_vacant(count: i64) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::person_id.is_null())
            .filter(roles::active.eq(true))
            .limit(count)
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Returns vacant and active roles by a team_id
    pub fn get_vacant_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .filter(roles::person_id.is_null())
            .filter(roles::active.eq(true))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Get roles by person ID. Can add a boolean to choose between active or inactive roles.
    pub fn get_by_person_id(id: Uuid, active: bool) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::person_id.eq(id))
            .filter(roles::active.eq(active))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }
    
    pub fn update(&mut self) -> Result<Self> {
        let mut conn = connection()?;

        self.updated_at = chrono::Utc::now().naive_utc();

        let res = diesel::update(roles::table)
        .filter(roles::id.eq(&self.id))
        .set(self.clone())
        .get_result(&mut conn)?;
        
        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub person_id: Option<Uuid>,
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    pub active: bool,
    // HR info - this will be another module - just here for expediency
    pub military_occupation: MilitaryOccupation,
    pub rank: Rank,
    pub start_datestamp: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
}

impl NewRole {

    pub fn new(
        person_id: Option<Uuid>,
        team_id: Uuid,
        title_en: String,
        title_fr: String,
        effort: f64,
        active: bool,
        military_occupation: MilitaryOccupation,
        rank: Rank,
        start_datestamp: NaiveDateTime,
        end_date: Option<NaiveDateTime>,
    ) -> Self {
        NewRole {
            person_id,
            team_id,
            title_en,
            title_fr,
            effort,
            active,
            military_occupation,
            rank,
            start_datestamp,
            end_date,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Enum, DbEnum, Copy, Display)]
#[ExistingTypePath = "crate::schema::sql_types::Rank"]
/// Represents military rank structure
pub enum Rank {
    Private,
    Corporal,
    MasterCorporal,
    Sergeant,
    WarrantOfficer,
    MasterWarrantOfficer,
    ChiefWarrantOfficer,
    SecondLieutenant,
    Lieutenant,
    Captain,
    Major,
    LieutenantColonel,
    Colonel,
    BrigadierGeneral,
    MajorGeneral,
    LieutenantGeneral,
    General,
}

impl Rank {
    /// Returns the next rank (promotion)
    /// Returns the same rank if already at the highest rank
    pub fn next(&self) -> Rank {
        match self {
            // Non-Commissioned Member progression
            Rank::Private => Rank::Corporal,
            Rank::Corporal => Rank::MasterCorporal,
            Rank::MasterCorporal => Rank::Sergeant,
            Rank::Sergeant => Rank::WarrantOfficer,
            Rank::WarrantOfficer => Rank::MasterWarrantOfficer,
            Rank::MasterWarrantOfficer => Rank::ChiefWarrantOfficer,
            Rank::ChiefWarrantOfficer => Rank::Lieutenant, // Highest NCM rank
            
            // Officer progression
            Rank::SecondLieutenant => Rank::Lieutenant,
            Rank::Lieutenant => Rank::Captain,
            Rank::Captain => Rank::Major,
            Rank::Major => Rank::LieutenantColonel,
            Rank::LieutenantColonel => Rank::Colonel,
            Rank::Colonel => Rank::BrigadierGeneral,
            Rank::BrigadierGeneral => Rank::MajorGeneral,
            Rank::MajorGeneral => Rank::LieutenantGeneral,
            Rank::LieutenantGeneral => Rank::General,
            Rank::General => Rank::General, // Highest rank
        }
    }

    /// Returns the previous rank (demotion)
    /// Returns the same rank if already at the lowest rank
    pub fn previous(&self) -> Rank {
        match self {
            // Non-Commissioned Member regression
            Rank::Private => Rank::Private, // Lowest rank
            Rank::Corporal => Rank::Private,
            Rank::MasterCorporal => Rank::Corporal,
            Rank::Sergeant => Rank::MasterCorporal,
            Rank::WarrantOfficer => Rank::Sergeant,
            Rank::MasterWarrantOfficer => Rank::WarrantOfficer,
            Rank::ChiefWarrantOfficer => Rank::MasterWarrantOfficer,
            
            // Officer regression
            Rank::SecondLieutenant => Rank::SecondLieutenant, // Lowest officer rank
            Rank::Lieutenant => Rank::SecondLieutenant,
            Rank::Captain => Rank::Lieutenant,
            Rank::Major => Rank::Captain,
            Rank::LieutenantColonel => Rank::Major,
            Rank::Colonel => Rank::LieutenantColonel,
            Rank::BrigadierGeneral => Rank::Colonel,
            Rank::MajorGeneral => Rank::BrigadierGeneral,
            Rank::LieutenantGeneral => Rank::MajorGeneral,
            Rank::General => Rank::LieutenantGeneral,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Enum, DbEnum, Copy, Display)]
#[ExistingTypePath = "crate::schema::sql_types::MilitaryOccupation"]
/// Represents Canadian Army military occupations
pub enum MilitaryOccupation {
    Infantry,
    Armoured,
    Artillery,
    CombatEngineers,
    Signals,
    Intelligence,
    MilitaryPolice,
    LogisticsSupport,
    MedicalTechnician,
    Communications,
    Electronics,
    VehicleTechnician,
    WeaponsTechnician,
    SupplyTechnician,
    CookSupport,
    FinanceClerk,
    HumanResourcesAdministrator,
    MilitaryFirefighter,
    MaterialsManagement,
    GeomaticsTechnician,
    MedicalAssistant,
    DentalAssistant,
    PharmacyTechnician,
    Chaplain,
    LegalOfficer,
    Pilot,
    AircrewSystems,
    AirTrafficController,
    WeatherTechnician,
    ImageTechnician,
    Musician,
    PhysicalFitnessInstructor,
    Cyber,
    SpecialForces,
    Officer,
    Other,
}

impl MilitaryOccupation {
    pub fn choose() -> MilitaryOccupation {
        let choice: MilitaryOccupation = rand::random();
        choice
    }
}

impl Distribution<Rank> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rank {
        match rng.gen_range(0..100) {
            0..=24 => Rank::Private,          // 25% - Entry level
            25..=44 => Rank::Corporal,        // 20% - Junior NCO
            45..=59 => Rank::MasterCorporal,  // 15% - Senior NCO
            60..=74 => Rank::Sergeant,        // 15% - Staff NCO
            75..=84 => Rank::WarrantOfficer,  // 10% - Warrant Officers
            85..=89 => Rank::MasterWarrantOfficer,     // 5%
            90..=92 => Rank::ChiefWarrantOfficer,      // 3%
            93..=95 => Rank::SecondLieutenant,         // 3% - Junior Officers
            96..=97 => Rank::Lieutenant,               // 2%
            98 => Rank::Captain,                       // 1%
            99 => Rank::Major,                         // 1% - Senior Officers
            100 => Rank::LieutenantColonel,            // <1%
            101 => Rank::Colonel,                      // <1%
            102 => Rank::BrigadierGeneral,             // <1% - Flag Officers
            103 => Rank::MajorGeneral,                 // <1%
            104 => Rank::LieutenantGeneral,            // <1%
            _ => Rank::General,                        // <1%
        }
    }
}

impl Distribution<MilitaryOccupation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MilitaryOccupation {
        match rng.gen_range(0..100) {
            0..=14 => MilitaryOccupation::Infantry,           // 15% - Core combat role
            15..=24 => MilitaryOccupation::LogisticsSupport,  // 10% - Essential support
            25..=32 => MilitaryOccupation::Communications,    // 8% - Critical infrastructure
            33..=39 => MilitaryOccupation::VehicleTechnician, // 7% - Maintenance
            40..=46 => MilitaryOccupation::SupplyTechnician,  // 7% - Supply chain
            47..=52 => MilitaryOccupation::MedicalTechnician, // 6% - Healthcare
            53..=58 => MilitaryOccupation::Artillery,         // 6% - Fire support
            59..=63 => MilitaryOccupation::CombatEngineers,   // 5% - Engineering
            64..=68 => MilitaryOccupation::Signals,           // 5% - Communications
            69..=72 => MilitaryOccupation::Armoured,          // 4% - Armoured corps
            73..=76 => MilitaryOccupation::Intelligence,      // 4% - Intel gathering
            77..=80 => MilitaryOccupation::Electronics,       // 4% - Technical support
            81..=83 => MilitaryOccupation::WeaponsTechnician, // 3% - Weapons maintenance
            84..=86 => MilitaryOccupation::MilitaryPolice,    // 3% - Security
            87..=89 => MilitaryOccupation::FinanceClerk,      // 3% - Administration
            90..=91 => MilitaryOccupation::HumanResourcesAdministrator, // 2%
            92 => MilitaryOccupation::CookSupport,       // 1% - Food services
            93 => MilitaryOccupation::Officer,          // 1% - Officer
            94 => MilitaryOccupation::MedicalAssistant,       // 1%
            95 => MilitaryOccupation::GeomaticsTechnician,    // 1%
            96 => MilitaryOccupation::MilitaryFirefighter,    // 1%
            97 => MilitaryOccupation::Cyber,                  // 1% - Emerging field
            98 => MilitaryOccupation::Pilot,                  // 1% - Specialized
            99 => MilitaryOccupation::SpecialForces,          // 1% - Elite units
            _ => MilitaryOccupation::Other,                   // <1% - Miscellaneous
        }
    }
}

pub fn find_people_by_requirements_met(requirements: Vec<Requirement>) -> Result<Vec<Person>> {

    let mut people_ids = Vec::new();

    let num_matches_required = *&requirements.len() as i32;

    for req in requirements {

        let caps = Capability::get_by_skill_id_and_level(req.skill_id, req.required_level)?;

        for c in caps {
            people_ids.push(c.person_id);
        };
    }

    let id_counts: HashMap<Uuid, i32> =
        people_ids.iter()
            .fold(HashMap::new(), |mut map, id| {
                *map.entry(*id).or_insert(0) += 1;
                map
            });

    let mut validated_ids: Vec<Uuid> = Vec::new();

    for (k, v) in id_counts {
        if v >= num_matches_required {
            validated_ids.push(k);
        }
    };


    Person::get_by_ids(&validated_ids)
}
