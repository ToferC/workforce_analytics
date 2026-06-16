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

        // ── Software Engineering (11 → 7, language variants merged) ────────
        (SkillDomain::SoftwareEngineering, "Software Engineering",                 "Génie logiciel"),
        (SkillDomain::SoftwareEngineering, "Front-end & UI Development",           "Développement frontal et interface utilisateur"),
        (SkillDomain::SoftwareEngineering, "Back-end & Services Development",      "Développement dorsal et services"),
        (SkillDomain::SoftwareEngineering, "Mobile Development",                   "Développement mobile"),
        (SkillDomain::SoftwareEngineering, "API & Integration Design",             "Conception d'API et d'intégration"),
        (SkillDomain::SoftwareEngineering, "Distributed Systems Architecture",     "Architecture de systèmes distribués"),
        (SkillDomain::SoftwareEngineering, "Database Engineering",                 "Ingénierie des bases de données"),

        // ── Cloud, Platform & DevOps (19 → 7, providers/practices merged) ──
        (SkillDomain::CloudPlatformDevOps, "Cloud Architecture & Administration",              "Architecture et administration infonuagiques"),
        (SkillDomain::CloudPlatformDevOps, "Container & Orchestration Technologies",           "Technologies de conteneurs et d'orchestration"),
        (SkillDomain::CloudPlatformDevOps, "Platform Engineering & Infrastructure as Code",    "Ingénierie de plateforme et infrastructure en tant que code"),
        (SkillDomain::CloudPlatformDevOps, "Networking & Infrastructure",                      "Réseautique et infrastructure"),
        (SkillDomain::CloudPlatformDevOps, "CI/CD & Release Engineering",                      "Intégration continue et ingénierie des livraisons"),
        (SkillDomain::CloudPlatformDevOps, "Site Reliability & Observability",                 "Fiabilité des sites et observabilité"),
        (SkillDomain::CloudPlatformDevOps, "Enterprise & Solution Architecture",               "Architecture d'entreprise et de solutions"),

        // ── Data, Analytics & AI (12 → 7, governance/BI/ML merged) ─────────
        (SkillDomain::DataAnalyticsAndAi, "Data Engineering & Pipelines",          "Ingénierie des données et pipelines"),
        (SkillDomain::DataAnalyticsAndAi, "Data Modelling & Architecture",         "Modélisation et architecture des données"),
        (SkillDomain::DataAnalyticsAndAi, "Data Governance & Quality",             "Gouvernance et qualité des données"),
        (SkillDomain::DataAnalyticsAndAi, "Business Intelligence & Visualisation","Intelligence d'affaires et visualisation"),
        (SkillDomain::DataAnalyticsAndAi, "Statistical & Quantitative Analysis",   "Analyse statistique et quantitative"),
        (SkillDomain::DataAnalyticsAndAi, "Machine Learning & AI Engineering",     "Ingénierie de l'apprentissage automatique et de l'IA"),
        (SkillDomain::DataAnalyticsAndAi, "Geospatial Analysis",                   "Analyse géospatiale"),

        // ── Cyber Security (10 → 7, engineering/offensive/governance merged) ─
        (SkillDomain::CyberSecurity, "Security Engineering & Architecture",                "Ingénierie et architecture de sécurité"),
        (SkillDomain::CyberSecurity, "Offensive Security & Vulnerability Management",      "Sécurité offensive et gestion des vulnérabilités"),
        (SkillDomain::CyberSecurity, "Identity & Access Management",                       "Gestion des identités et des accès"),
        (SkillDomain::CyberSecurity, "Security Operations & Incident Response",            "Opérations de sécurité et réponse aux incidents"),
        (SkillDomain::CyberSecurity, "Information Security Governance & Risk",             "Gouvernance et risque de sécurité de l'information"),
        (SkillDomain::CyberSecurity, "Cloud & Infrastructure Security",                    "Sécurité infonuagique et de l'infrastructure"),
        (SkillDomain::CyberSecurity, "Cryptography & PKI",                                 "Cryptographie et ICP"),

        // ── Product, Agile & Delivery (14 → 7, PM + Agile/Delivery merged) ─
        (SkillDomain::ProductAgileAndDelivery, "Product Strategy & Vision",                       "Stratégie et vision produit"),
        (SkillDomain::ProductAgileAndDelivery, "Roadmapping, Prioritisation & Backlog Management","Feuille de route, priorisation et gestion du carnet de travail"),
        (SkillDomain::ProductAgileAndDelivery, "Agile Delivery & Coaching",                       "Livraison agile et coaching"),
        (SkillDomain::ProductAgileAndDelivery, "Programme & Project Management",                  "Gestion de programmes et de projets"),
        (SkillDomain::ProductAgileAndDelivery, "Market & User Research and Stakeholder Engagement","Recherche marché et utilisateurs et engagement des parties prenantes"),
        (SkillDomain::ProductAgileAndDelivery, "Business Case, Benefits & Outcome Measurement",   "Analyse de rentabilisation, gestion des avantages et mesure des résultats"),
        (SkillDomain::ProductAgileAndDelivery, "Delivery Risk & Dependency Management",           "Gestion des risques et des dépendances de livraison"),

        // ── User Experience & Design (10 → 7, research/IA/prototyping merged) ─
        (SkillDomain::UserExperience, "User Research & Usability Testing",     "Recherche utilisateurs et tests d'utilisabilité"),
        (SkillDomain::UserExperience, "Service Design",                        "Conception de services"),
        (SkillDomain::UserExperience, "Interaction & Information Design",      "Conception d'interaction et de l'information"),
        (SkillDomain::UserExperience, "Prototyping & Design Systems",          "Prototypage et systèmes de conception"),
        (SkillDomain::UserExperience, "Accessibility & Inclusive Design",      "Accessibilité et conception inclusive"),
        (SkillDomain::UserExperience, "Content Design",                        "Conception de contenu"),
        (SkillDomain::UserExperience, "UX Strategy",                           "Stratégie UX"),

        // ── Procurement & Vendor Management (17 → 7, lifecycle stages) ─────
        (SkillDomain::ProcurementAndVendorManagement, "Procurement Strategy & Sourcing",            "Stratégie d'approvisionnement et sourçage"),
        (SkillDomain::ProcurementAndVendorManagement, "Solicitation Development (SOW & RFx)",       "Élaboration de documents de sollicitation"),
        (SkillDomain::ProcurementAndVendorManagement, "Bid Evaluation & Contract Negotiation",      "Évaluation des soumissions et négociation de contrats"),
        (SkillDomain::ProcurementAndVendorManagement, "Contract Administration & Compliance",       "Administration des contrats et conformité"),
        (SkillDomain::ProcurementAndVendorManagement, "Vendor Relationship & Performance Management","Gestion des relations et de la performance des fournisseurs"),
        (SkillDomain::ProcurementAndVendorManagement, "Commercial & Risk Analysis",                 "Analyse commerciale et des risques"),
        (SkillDomain::ProcurementAndVendorManagement, "Vendor & Asset Lifecycle Management",        "Gestion du cycle de vie des fournisseurs et des actifs"),

        // ── People & Organisational Leadership (GoC Key Leadership Competencies) ─
        (SkillDomain::PeopleAndOrganisationalLeadership, "Create Vision and Strategy",                "Créer une vision et une orientation stratégiques"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Mobilize People",                           "Mobiliser les personnes"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Uphold Integrity and Respect",              "Promouvoir l'intégrité et le respect"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Collaborate with Partners and Stakeholders","Collaborer avec les partenaires et les intervenants"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Promote Innovation and Guide Change",       "Promouvoir l'innovation et orienter le changement"),
        (SkillDomain::PeopleAndOrganisationalLeadership, "Achieve Results",                           "Obtenir des résultats"),

        // ── Governance & Compliance (18 → 7, capability clusters) ──────────
        (SkillDomain::Governance, "IT & Digital Governance",                       "Gouvernance des TI et du numérique"),
        (SkillDomain::Governance, "Enterprise Risk & Business Continuity",         "Gestion des risques organisationnels et continuité des activités"),
        (SkillDomain::Governance, "Audit & Programme Evaluation",                  "Vérification et évaluation des programmes"),
        (SkillDomain::Governance, "Privacy & Access to Information",               "Protection des renseignements personnels et accès à l'information"),
        (SkillDomain::Governance, "Information & Records Management",              "Gestion de l'information et des documents"),
        (SkillDomain::Governance, "Corporate Planning & Reporting",                "Planification et rapports organisationnels"),
        (SkillDomain::Governance, "Policy Development & Advice",                   "Élaboration de politiques et conseils stratégiques"),
        (SkillDomain::Governance, "Legal Affairs & Official Languages",            "Affaires juridiques et langues officielles"),

        // ── Corporate Services (18 → 7, corporate function clusters) ───────
        (SkillDomain::CorporateServices, "Financial Management & Reporting",       "Gestion financière et rapports"),
        (SkillDomain::CorporateServices, "Staffing & Talent Acquisition",          "Dotation et acquisition de talents"),
        (SkillDomain::CorporateServices, "Compensation & Benefits Administration", "Administration de la rémunération et des avantages"),
        (SkillDomain::CorporateServices, "Learning, Equity & Workplace Wellbeing", "Apprentissage, équité et bien-être au travail"),
        (SkillDomain::CorporateServices, "Employee & Labour Relations",            "Relations avec les employés et relations de travail"),
        (SkillDomain::CorporateServices, "Communications & Public Affairs",        "Communications et affaires publiques"),
        (SkillDomain::CorporateServices, "Administrative & Logistics Support",     "Soutien administratif et logistique"),
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

    for (i, c) in capabilities.into_iter().enumerate() {
        if i % 100 == 0 {
            print!(".")
        }

        // A single central authority validates the capability, setting the
        // validated level directly. Creating the validation also stamps the
        // capability with the authority and date for provenance.
        let authority: Uuid = person_ids
            .choose(&mut rng)
            .cloned()
            .expect("No people available to act as validation authority");

        let assessment = match rng.gen_range(0..10) {
            0..=3  => c.self_identified_level.step_down(),
            4..=6  => c.self_identified_level,
            _      => c.self_identified_level.step_up(),
        };

        let v = NewValidation::new(
            authority,
            c.id,
            assessment,
        );

        let _r = Validation::create(&v)?;
        progress.increment();
    }

    progress.done();

    Ok(())
}
