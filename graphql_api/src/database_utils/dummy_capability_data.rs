use rand::{seq::SliceRandom, Rng};
use uuid::Uuid;
use async_graphql::Error;
use crate::progress::progress::ProgressLogger;

use crate::models::{Affiliation, NewAffiliation, NewCapability, Capability, Skill, NewSkill, CapabilityLevel, SkillDomain,
    LanguageLevel, LanguageName, NewLanguageData, LanguageData, Person, NewValidation, Validation};

pub fn pre_populate_skills() -> Result<(), Error> {

    let skills: Vec<(SkillDomain, &str, &str)> = vec![

        // ── Combat Operations ──────────────────────────────────────────────
        (SkillDomain::Combat, "Infantry Operations",           "Opérations d'infanterie"),
        (SkillDomain::Combat, "Armoured Operations",           "Opérations blindées"),
        (SkillDomain::Combat, "Artillery Operations",          "Opérations d'artillerie"),
        (SkillDomain::Combat, "Close Quarters Combat",         "Combat rapproché"),
        (SkillDomain::Combat, "Marksmanship",                  "Tir de précision"),
        (SkillDomain::Combat, "Small Unit Tactics",            "Tactiques de petites unités"),
        (SkillDomain::Combat, "Battlefield Awareness",         "Conscience situationnelle au combat"),
        (SkillDomain::Combat, "Weapons Systems Employment",    "Emploi des systèmes d'armes"),
        (SkillDomain::Combat, "Urban Operations",              "Opérations en milieu urbain"),

        // ── Intelligence & Reconnaissance ──────────────────────────────────
        (SkillDomain::Intelligence, "Intelligence Collection Planning",  "Planification de la collecte du renseignement"),
        (SkillDomain::Intelligence, "Reconnaissance",                    "Reconnaissance"),
        (SkillDomain::Intelligence, "Surveillance",                      "Surveillance"),
        (SkillDomain::Intelligence, "Signals Intelligence",              "Renseignement d'origine électromagnétique"),
        (SkillDomain::Intelligence, "Human Intelligence",                "Renseignement d'origine humaine"),
        (SkillDomain::Intelligence, "Threat Assessment",                 "Évaluation des menaces"),
        (SkillDomain::Intelligence, "All-Source Intelligence Analysis",  "Analyse de renseignement toutes sources"),
        (SkillDomain::Intelligence, "Intelligence Reporting",            "Rapports de renseignement"),

        // ── Strategy & Operational Planning ────────────────────────────────
        (SkillDomain::Strategy, "Strategic Planning",       "Planification stratégique"),
        (SkillDomain::Strategy, "Military Doctrine",        "Doctrine militaire"),
        (SkillDomain::Strategy, "Operational Planning",     "Planification opérationnelle"),
        (SkillDomain::Strategy, "Tactical Analysis",        "Analyse tactique"),
        (SkillDomain::Strategy, "Force Structure Planning", "Planification de la structure des forces"),
        (SkillDomain::Strategy, "Mission Analysis",         "Analyse de la mission"),
        (SkillDomain::Strategy, "Campaign Planning",        "Planification de la campagne"),
        (SkillDomain::Strategy, "Organisational Strategy",  "Stratégie organisationnelle"),

        // ── Military Engineering ────────────────────────────────────────────
        (SkillDomain::Engineering, "Explosive Ordnance Disposal",    "Neutralisation des engins explosifs"),
        (SkillDomain::Engineering, "Combat Engineering",             "Génie de combat"),
        (SkillDomain::Engineering, "Fortification & Field Defences", "Fortification et défenses de campagne"),
        (SkillDomain::Engineering, "Military Bridge Construction",   "Construction de ponts militaires"),
        (SkillDomain::Engineering, "Demolitions",                    "Démolitions"),
        (SkillDomain::Engineering, "Mine Warfare",                   "Guerre des mines"),
        (SkillDomain::Engineering, "Technical Intelligence",         "Renseignement technique"),

        // ── Medical & Health Services ───────────────────────────────────────
        (SkillDomain::Medical, "Combat Medicine",        "Médecine de combat"),
        (SkillDomain::Medical, "Battlefield Trauma Care","Soins aux traumatisés de combat"),
        (SkillDomain::Medical, "Field Surgery",          "Chirurgie de campagne"),
        (SkillDomain::Medical, "Emergency Medicine",     "Médecine d'urgence"),
        (SkillDomain::Medical, "Triage",                 "Triage"),
        (SkillDomain::Medical, "Medical Evacuation",     "Évacuation médicale"),
        (SkillDomain::Medical, "CBRN Medical Response",  "Réponse médicale CBRN"),

        // ── Joint & Coalition Operations ────────────────────────────────────
        (SkillDomain::JointOperations, "Joint Operations",                  "Opérations interarmées"),
        (SkillDomain::JointOperations, "Coalition Warfare",                 "Guerre de coalition"),
        (SkillDomain::JointOperations, "Multinational Staff Work",          "Travail d'état-major multinational"),
        (SkillDomain::JointOperations, "Military Interoperability",         "Interopérabilité militaire"),
        (SkillDomain::JointOperations, "Combined Arms Integration",         "Intégration des armes combinées"),
        (SkillDomain::JointOperations, "Liaison & Coordination",            "Liaison et coordination"),
        (SkillDomain::JointOperations, "International Military Cooperation","Coopération militaire internationale"),

        // ── Software Engineering ────────────────────────────────────────────
        (SkillDomain::SoftwareEngineering, "Software Engineering – Python",        "Génie logiciel – Python"),
        (SkillDomain::SoftwareEngineering, "Software Engineering – TypeScript",    "Génie logiciel – TypeScript"),
        (SkillDomain::SoftwareEngineering, "Software Engineering – Rust",          "Génie logiciel – Rust"),
        (SkillDomain::SoftwareEngineering, "Software Engineering – Go",            "Génie logiciel – Go"),
        (SkillDomain::SoftwareEngineering, "Software Engineering – Java/Kotlin",   "Génie logiciel – Java/Kotlin"),
        (SkillDomain::SoftwareEngineering, "Front-end Development",                "Développement frontal"),
        (SkillDomain::SoftwareEngineering, "Back-end Development",                 "Développement dorsal"),
        (SkillDomain::SoftwareEngineering, "Mobile Development",                   "Développement mobile"),
        (SkillDomain::SoftwareEngineering, "API Design",                           "Conception d'API"),
        (SkillDomain::SoftwareEngineering, "Microservices & Distributed Systems",  "Microservices et systèmes distribués"),
        (SkillDomain::SoftwareEngineering, "Database Engineering",                 "Ingénierie des bases de données"),

        // ── Cloud, Platform & DevOps ────────────────────────────────────────
        (SkillDomain::CloudPlatformDevOps, "Cloud Architecture – AWS",            "Architecture infonuagique – AWS"),
        (SkillDomain::CloudPlatformDevOps, "Cloud Architecture – Azure",          "Architecture infonuagique – Azure"),
        (SkillDomain::CloudPlatformDevOps, "Cloud Architecture – GCP",            "Architecture infonuagique – GCP"),
        (SkillDomain::CloudPlatformDevOps, "Cloud Administration",                "Administration infonuagique"),
        (SkillDomain::CloudPlatformDevOps, "Container Technologies",              "Technologies de conteneurs"),
        (SkillDomain::CloudPlatformDevOps, "Container Orchestration – Kubernetes","Orchestration – Kubernetes"),
        (SkillDomain::CloudPlatformDevOps, "Networking & Infrastructure",         "Réseautique et infrastructure"),
        (SkillDomain::CloudPlatformDevOps, "Platform Engineering",                "Ingénierie de plateforme"),
        (SkillDomain::CloudPlatformDevOps, "CI/CD Pipeline Engineering",          "Ingénierie de pipelines IC/DC"),
        (SkillDomain::CloudPlatformDevOps, "Infrastructure as Code",              "Infrastructure en tant que code"),
        (SkillDomain::CloudPlatformDevOps, "Site Reliability Engineering",        "Ingénierie de la fiabilité des sites"),
        (SkillDomain::CloudPlatformDevOps, "Monitoring & Observability",          "Surveillance et observabilité"),
        (SkillDomain::CloudPlatformDevOps, "Configuration Management",            "Gestion des configurations"),
        (SkillDomain::CloudPlatformDevOps, "Release Engineering",                 "Ingénierie des livraisons"),
        (SkillDomain::CloudPlatformDevOps, "FinOps",                              "FinOps"),
        (SkillDomain::CloudPlatformDevOps, "Enterprise Architecture",             "Architecture d'entreprise"),
        (SkillDomain::CloudPlatformDevOps, "Solution Architecture",               "Architecture de solutions"),
        (SkillDomain::CloudPlatformDevOps, "Integration Architecture",            "Architecture d'intégration"),
        (SkillDomain::CloudPlatformDevOps, "Legacy Modernisation",                "Modernisation des systèmes patrimoniaux"),

        // ── Data, Analytics & AI ────────────────────────────────────────────
        (SkillDomain::DataAnalyticsAndAi, "Data Engineering",              "Ingénierie des données"),
        (SkillDomain::DataAnalyticsAndAi, "Data Modelling",                "Modélisation des données"),
        (SkillDomain::DataAnalyticsAndAi, "Data Governance",               "Gouvernance des données"),
        (SkillDomain::DataAnalyticsAndAi, "Data Quality Management",       "Gestion de la qualité des données"),
        (SkillDomain::DataAnalyticsAndAi, "Business Intelligence & Reporting","Intelligence d'affaires et rapports"),
        (SkillDomain::DataAnalyticsAndAi, "Data Visualisation",            "Visualisation des données"),
        (SkillDomain::DataAnalyticsAndAi, "Statistical Analysis",          "Analyse statistique"),
        (SkillDomain::DataAnalyticsAndAi, "Machine Learning Engineering",  "Ingénierie de l'apprentissage automatique"),
        (SkillDomain::DataAnalyticsAndAi, "AI & LLM Integration",          "Intégration IA et grands modèles de langage"),
        (SkillDomain::DataAnalyticsAndAi, "MLOps",                         "MLOps"),
        (SkillDomain::DataAnalyticsAndAi, "Geospatial Analysis",           "Analyse géospatiale"),
        (SkillDomain::DataAnalyticsAndAi, "Operations Research",           "Recherche opérationnelle"),

        // ── Cyber Security ──────────────────────────────────────────────────
        (SkillDomain::CyberSecurity, "Security Engineering",            "Ingénierie de la sécurité"),
        (SkillDomain::CyberSecurity, "Threat Modelling",                "Modélisation des menaces"),
        (SkillDomain::CyberSecurity, "Penetration Testing",             "Test de pénétration"),
        (SkillDomain::CyberSecurity, "Vulnerability Management",        "Gestion des vulnérabilités"),
        (SkillDomain::CyberSecurity, "Identity & Access Management",    "Gestion des identités et des accès"),
        (SkillDomain::CyberSecurity, "Security Operations",             "Opérations de sécurité"),
        (SkillDomain::CyberSecurity, "Incident Response",               "Réponse aux incidents"),
        (SkillDomain::CyberSecurity, "Information Security Governance", "Gouvernance de la sécurité de l'information"),
        (SkillDomain::CyberSecurity, "Cloud Security Posture Management","Gestion de la posture de sécurité infonuagique"),
        (SkillDomain::CyberSecurity, "Cryptography & PKI",              "Cryptographie et ICP"),

        // ── Product Management ──────────────────────────────────────────────
        (SkillDomain::ProductManagement, "Product Strategy",             "Stratégie produit"),
        (SkillDomain::ProductManagement, "Product Roadmapping",          "Feuille de route produit"),
        (SkillDomain::ProductManagement, "Backlog Management",           "Gestion du carnet de travail"),
        (SkillDomain::ProductManagement, "User Story Writing",           "Rédaction de récits utilisateurs"),
        (SkillDomain::ProductManagement, "Market & Competitive Research","Recherche marché et concurrentielle"),
        (SkillDomain::ProductManagement, "Stakeholder Engagement",       "Engagement des parties prenantes"),
        (SkillDomain::ProductManagement, "Business Case Development",    "Élaboration du dossier d'analyse de rentabilisation"),
        (SkillDomain::ProductManagement, "OKR & Outcome Measurement",    "Mesure des résultats et des OKR"),
        (SkillDomain::ProductManagement, "Benefits Realisation",         "Réalisation des avantages"),
        (SkillDomain::ProductManagement, "Portfolio Prioritisation",     "Priorisation du portefeuille"),

        // ── Agile & Delivery ────────────────────────────────────────────────
        (SkillDomain::AgileAndDelivery, "Scrum",                     "Scrum"),
        (SkillDomain::AgileAndDelivery, "Kanban",                    "Kanban"),
        (SkillDomain::AgileAndDelivery, "SAFe – Scaled Agile Framework","SAFe – cadre agile à grande échelle"),
        (SkillDomain::AgileAndDelivery, "Lean Thinking",             "Pensée Lean"),
        (SkillDomain::AgileAndDelivery, "Delivery Management",       "Gestion de la livraison"),
        (SkillDomain::AgileAndDelivery, "Programme Management",      "Gestion de programme"),
        (SkillDomain::AgileAndDelivery, "Project Management",        "Gestion de projet"),
        (SkillDomain::AgileAndDelivery, "Agile Coaching",            "Coaching agile"),
        (SkillDomain::AgileAndDelivery, "Retrospective Facilitation","Animation de rétrospectives"),
        (SkillDomain::AgileAndDelivery, "Delivery Risk Management",  "Gestion des risques de livraison"),

        // ── User Experience & Design ────────────────────────────────────────
        (SkillDomain::UserExperience, "User Research",               "Recherche utilisateurs"),
        (SkillDomain::UserExperience, "Usability Testing",           "Tests d'utilisabilité"),
        (SkillDomain::UserExperience, "Service Design",              "Conception de services"),
        (SkillDomain::UserExperience, "Interaction Design",          "Conception d'interaction"),
        (SkillDomain::UserExperience, "Information Architecture",    "Architecture de l'information"),
        (SkillDomain::UserExperience, "Prototyping & Wireframing",   "Prototypage et maquettage"),
        (SkillDomain::UserExperience, "Design Systems",              "Systèmes de conception"),
        (SkillDomain::UserExperience, "Accessibility & Inclusive Design","Accessibilité et conception inclusive"),
        (SkillDomain::UserExperience, "Content Design",              "Conception de contenu"),
        (SkillDomain::UserExperience, "UX Strategy",                 "Stratégie UX"),

        // ── Procurement & Vendor Management ────────────────────────────────
        (SkillDomain::ProcurementAndVendorManagement, "Procurement Strategy & Planning",     "Stratégie et planification des achats"),
        (SkillDomain::ProcurementAndVendorManagement, "Government Contracting Frameworks",   "Cadres d'approvisionnement gouvernementaux"),
        (SkillDomain::ProcurementAndVendorManagement, "Statement of Work Development",       "Élaboration de l'énoncé des travaux"),
        (SkillDomain::ProcurementAndVendorManagement, "RFx Development",                     "Élaboration de demandes de soumissions"),
        (SkillDomain::ProcurementAndVendorManagement, "Bid Evaluation & Selection",          "Évaluation des soumissions et sélection"),
        (SkillDomain::ProcurementAndVendorManagement, "Contract Negotiation",                "Négociation de contrats"),
        (SkillDomain::ProcurementAndVendorManagement, "Contract Administration",             "Administration des contrats"),
        (SkillDomain::ProcurementAndVendorManagement, "Strategic Sourcing",                  "Approvisionnement stratégique"),
        (SkillDomain::ProcurementAndVendorManagement, "Indigenous & Diversity Procurement",  "Approvisionnement autochtone et diversifié"),
        (SkillDomain::ProcurementAndVendorManagement, "Procurement Analytics",               "Analytique des achats"),
        (SkillDomain::ProcurementAndVendorManagement, "Vendor Relationship Management",      "Gestion des relations fournisseurs"),
        (SkillDomain::ProcurementAndVendorManagement, "SLA & Vendor Performance Management", "Gestion des niveaux de service et de la performance fournisseur"),
        (SkillDomain::ProcurementAndVendorManagement, "Third-Party Risk Management",         "Gestion des risques liés aux tiers"),
        (SkillDomain::ProcurementAndVendorManagement, "Vendor Onboarding & Offboarding",     "Intégration et départ des fournisseurs"),
        (SkillDomain::ProcurementAndVendorManagement, "Commercial & Pricing Analysis",       "Analyse commerciale et tarifaire"),
        (SkillDomain::ProcurementAndVendorManagement, "Software & Asset Lifecycle Management","Gestion du cycle de vie des logiciels et des actifs"),
        (SkillDomain::ProcurementAndVendorManagement, "Multi-Vendor Coordination",           "Coordination multi-fournisseurs"),

        // ── People & Organisational Leadership ─────────────────────────────
        (SkillDomain::PeopleAndOrganisationalLeadership, "Vision Setting",                   "Établissement de la vision"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Strategic Thinking",               "Réflexion stratégique"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Innovation",                       "Innovation"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Digital Leadership",               "Leadership numérique"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Political Acuity",                 "Acuité politique"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "People Leadership",                "Leadership des personnes"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Inclusive Leadership",             "Leadership inclusif"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Servant Leadership",               "Leadership serviteur"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Mobilizing People",                "Mobilisation des personnes"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Psychological Safety",             "Sécurité psychologique"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Building High-Performing Teams",   "Constitution d'équipes performantes"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Talent Development & Coaching",    "Développement des talents et coaching"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Performance Management",           "Gestion du rendement"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Employee Wellbeing",               "Bien-être des employés"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Executive Communication",          "Communication exécutive"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Negotiation",                      "Négociation"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Conflict Resolution",              "Résolution des conflits"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Organisational Design",            "Conception organisationnelle"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Workforce Planning",               "Planification des effectifs"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Resource & Capacity Management",   "Gestion des ressources et de la capacité"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Organisational Change Management", "Gestion du changement organisationnel"),

        // ── Governance & Compliance ─────────────────────────────────────────
        (SkillDomain::Governance, "IT Governance",                           "Gouvernance des TI"),
        (SkillDomain::Governance, "Digital Policy Development",              "Élaboration de politiques numériques"),
        (SkillDomain::Governance, "Enterprise Risk Management",              "Gestion intégrée des risques"),
        (SkillDomain::Governance, "IT Audit & Assurance",                    "Vérification et assurance des TI"),
        (SkillDomain::Governance, "Business Continuity & Disaster Recovery", "Continuité des activités et reprise après sinistre"),
        (SkillDomain::Governance, "IT Change Governance",                    "Gouvernance des changements TI"),
        (SkillDomain::Governance, "Privacy & Access to Information (ATIP)",  "Protection des renseignements personnels et AIPRP"),
        (SkillDomain::Governance, "Privacy Impact Assessment",               "Évaluation des facteurs relatifs à la vie privée"),
        (SkillDomain::Governance, "Information Management",                  "Gestion de l'information"),
        (SkillDomain::Governance, "Records Management",                      "Gestion des documents"),
        (SkillDomain::Governance, "Security Classification Management",      "Gestion de la classification de sécurité"),
        (SkillDomain::Governance, "Open Government & Transparency",          "Gouvernement ouvert et transparence"),
        (SkillDomain::Governance, "Corporate Planning & Performance Reporting","Planification et rapports de performance organisationnels"),
        (SkillDomain::Governance, "Treasury Board Submissions",              "Présentations au Conseil du Trésor"),
        (SkillDomain::Governance, "Internal Audit",                          "Vérification interne"),
        (SkillDomain::Governance, "Programme Evaluation",                    "Évaluation des programmes"),
        (SkillDomain::Governance, "Legal & Policy Interpretation",           "Interprétation juridique et des politiques"),
        (SkillDomain::Governance, "Official Languages Management",           "Gestion des langues officielles"),

        // ── Corporate Services ──────────────────────────────────────────────
        (SkillDomain::CorporateServices, "Accounting",                     "Comptabilité"),
        (SkillDomain::CorporateServices, "Financial Forecasting",          "Prévision financière"),
        (SkillDomain::CorporateServices, "Government Budgeting",           "Budgétisation gouvernementale"),
        (SkillDomain::CorporateServices, "Financial Analysis & Reporting", "Analyse et rapports financiers"),
        (SkillDomain::CorporateServices, "Staffing",                       "Dotation en personnel"),
        (SkillDomain::CorporateServices, "Classification",                 "Classification"),
        (SkillDomain::CorporateServices, "Recruiting & Talent Acquisition","Recrutement et acquisition de talents"),
        (SkillDomain::CorporateServices, "Pay & Compensation",             "Rémunération et avantages sociaux"),
        (SkillDomain::CorporateServices, "Learning & Development",         "Apprentissage et perfectionnement"),
        (SkillDomain::CorporateServices, "Equity, Diversity & Inclusion",  "Équité, diversité et inclusion"),
        (SkillDomain::CorporateServices, "Employee Relations",             "Relations avec les employés"),
        (SkillDomain::CorporateServices, "Occupational Health & Safety",   "Santé et sécurité au travail"),
        (SkillDomain::CorporateServices, "Writing & Editing",              "Rédaction et révision"),
        (SkillDomain::CorporateServices, "Public Speaking & Presentation", "Prise de parole et présentation"),
        (SkillDomain::CorporateServices, "Media Relations",                "Relations avec les médias"),
        (SkillDomain::CorporateServices, "Strategic Communications",       "Communications stratégiques"),
        (SkillDomain::CorporateServices, "Travel & Logistics Administration","Administration des voyages et de la logistique"),
        (SkillDomain::CorporateServices, "Administrative Operations",      "Opérations administratives"),
    ];

    for (domain, name_en, name_fr) in &skills {
        let ns = NewSkill::new(
            name_en.to_string(),
            name_fr.to_string(),
            *domain,
            None,
            None,
        );
        let _res = Skill::create(&ns)?;
    }

    Ok(())
}

pub fn create_fake_capabilities(
    people_ids: &Vec<Uuid>,
    org_id: Uuid,
    science_org_ids: &Vec<Uuid>,
) -> Result<(), Error> {

    let mut rng = rand::thread_rng();

    let mut capabilities = Vec::new();
    let mut language_datas = Vec::new();

    for person_id in people_ids {

        let science_org_id = science_org_ids.choose(&mut rng).unwrap();

        // Create LanguageDatas
        let primary_language = vec![LanguageName::English, LanguageName::French]
            .choose(&mut rng).unwrap().clone();

        let secondary_language = match primary_language {
            LanguageName::English => LanguageName::French,
            LanguageName::French  => LanguageName::English,
            _                     => LanguageName::English,
        };

        let primary = NewLanguageData::new(
            *person_id,
            primary_language,
            Some(LanguageLevel::E),
            Some(LanguageLevel::E),
            Some(LanguageLevel::E),
        );

        language_datas.push(primary);

        if rng.gen_bool(0.5) {
            let beginner     = vec![LanguageLevel::B, LanguageLevel::A, LanguageLevel::A];
            let intermediate = vec![LanguageLevel::C, LanguageLevel::B, LanguageLevel::B];
            let professional = vec![LanguageLevel::C, LanguageLevel::B, LanguageLevel::C];
            let fluent       = vec![LanguageLevel::E, LanguageLevel::E, LanguageLevel::E];

            let chosen = match rng.gen_range(0..=10) {
                0..=3  => beginner,
                4..=6  => intermediate,
                7..=9  => professional,
                10     => fluent,
                _      => beginner,
            };

            let secondary = NewLanguageData::new(
                *person_id,
                secondary_language,
                Some(chosen[0]),
                Some(chosen[1]),
                Some(chosen[2]),
            );

            language_datas.push(secondary);
        }

        // Choose two random domains for this person
        let mut sds: Vec<SkillDomain> = Vec::new();
        for _ in 0..2 {
            let sd: SkillDomain = rand::random();
            if !sds.contains(&sd) {
                sds.push(sd);
            }
        }

        // Research/tech professionals have a 20% chance of an academic affiliation
        let is_researcher = sds.contains(&SkillDomain::Engineering)
            || sds.contains(&SkillDomain::DataAnalyticsAndAi)
            || sds.contains(&SkillDomain::SoftwareEngineering);

        if is_researcher && rng.gen_bool(0.2) {
            let na = NewAffiliation::new(
                *person_id,
                *science_org_id,
                org_id,
                "Research Affiliate".to_string(),
                None,
            );
            let _res = Affiliation::create(&na)?;
        }

        for sd in sds {
            let skills_in_domain = Skill::get_by_domain(sd)?;

            let mut selected_skills: Vec<Skill> = Vec::new();
            for _ in 0..3 {
                let skill = skills_in_domain.choose(&mut rng).unwrap();
                if !selected_skills.contains(skill) {
                    selected_skills.push(skill.clone());
                }
            }

            let mut capability_level: CapabilityLevel = rand::random();

            for skill in selected_skills {
                let nc = NewCapability::new(
                    *person_id,
                    skill.id,
                    org_id,
                    capability_level,
                );

                capabilities.push(nc);

                if capabilities.len() > 1000 {
                    let _r = Capability::batch_create(&capabilities)?;
                    println!("Inserted {} capabilities", &capabilities.len());
                    capabilities = Vec::new();
                }

                capability_level = capability_level.step_down();
            }
        }
    }

    let _r = LanguageData::batch_create(language_datas)?;
    let _r = Capability::batch_create(&capabilities)?;

    Ok(())
}

pub fn create_validations() -> Result<(), Error> {

    println!("Adding validations to capabilities");

    let mut rng = rand::thread_rng();

    let person_ids   = Person::get_all_ids()?;
    let capabilities = Capability::get_all()?;

    let mut progress = ProgressLogger::new(
        "Adding validations to capabilities".to_owned(),
        capabilities.len(),
    );

    for (i, mut c) in capabilities.into_iter().enumerate() {
        let mut validations = Vec::new();

        let validators: Vec<Uuid> = person_ids
            .choose_multiple(&mut rng, 4)
            .cloned()
            .collect();

        if i % 100 == 0 {
            print!(".")
        }

        let mut validated_levels: Vec<CapabilityLevel> = Vec::new();

        for validator in validators {
            let assessment = match rng.gen_range(0..10) {
                0..=3  => c.self_identified_level.step_down(),
                4..=6  => c.self_identified_level,
                7..=9  => c.self_identified_level.step_up(),
                _      => c.self_identified_level.step_up(),
            };

            validated_levels.push(assessment);

            let v = NewValidation::new(
                validator,
                c.id,
                assessment,
            );

            validations.push(v.clone());
        }

        let _r = Validation::batch_create(validations)?;
        c.update_from_batch_validations(&validated_levels)?;
        progress.increment();
    }

    progress.done();

    Ok(())
}
