# Skills Framework: Digital & Technology Organisation (Defence Support)

**Purpose**: This document recommends additions to the `SkillDomain` enum and `Skill` seed data
to cover the range of capabilities needed by a civilian digital and technology organisation that
supports defence forces. It is intended for review before any code changes are made.

---

## Context

The existing `SkillDomain` enum and seed skills reflect a military force structure
(Combat, Strategy, Intelligence, Engineering, etc.). The current `InformationTechnology` domain
contains only seven skills (Cloud Admin, Cloud Architecture, Python, DB Admin, Networking,
Back-end Dev, Front-end Dev), and there are no domains for product management, user experience,
DevOps, or agile delivery.

A civilian digital and technology organisation supporting defence forces will typically employ:

- Software engineers, platform engineers, and site reliability engineers
- Data scientists, data engineers, and analysts
- Product managers and delivery managers
- UX researchers and service designers
- DevOps and cloud platform specialists
- Agile coaches and Scrum practitioners
- Cyber security specialists
- Enterprise and solution architects

The recommendations below expand the existing `InformationTechnology` domain and introduce
**six new `SkillDomain` values**.

---

## Recommended New `SkillDomain` Values

```rust
// Add to the SkillDomain enum in src/models/skill.rs
DataAndAnalytics,        // Data engineering, data science, ML, BI, governance
CyberSecurity,           // Security engineering, threat modelling, GRC
ProductManagement,       // Product ownership, roadmapping, stakeholder engagement
UserExperience,          // UX research, service design, accessibility
DevOpsAndPlatform,       // CI/CD, IaC, container orchestration, SRE
AgileDelivery,           // Scrum, Kanban, SAFe, delivery management
```

---

## Expanded `InformationTechnology` Domain

Replace the current seven IT skills with the following expanded set.

### Software Engineering

| name_en | name_fr | description_en |
|---|---|---|
| Software Engineering – Python | Génie logiciel – Python | Design and build production-quality Python applications including testing and packaging |
| Software Engineering – TypeScript / JavaScript | Génie logiciel – TypeScript / JavaScript | Develop front-end and Node.js applications using modern TypeScript and JavaScript ecosystems |
| Software Engineering – Rust | Génie logiciel – Rust | Build high-performance, memory-safe systems and services using Rust |
| Software Engineering – Go | Génie logiciel – Go | Write concurrent, cloud-native services and CLI tooling in Go |
| Software Engineering – Java / Kotlin | Génie logiciel – Java / Kotlin | Develop enterprise back-end services and Android applications using JVM languages |
| API Design & Integration | Conception d'API et intégration | Design REST, GraphQL, and gRPC APIs; implement service-to-service integration patterns |
| Microservices Architecture | Architecture de microservices | Decompose monolithic systems into independently deployable, loosely coupled services |
| Database Design & Administration | Conception et administration de bases de données | Model relational and document databases; manage migrations, tuning, and backup strategies |
| Front-end Development | Développement frontal | Build responsive, accessible web UIs using component frameworks such as React or Vue |
| Back-end Development | Développement dorsal | Implement server-side business logic, APIs, and data access layers |
| Mobile Development | Développement mobile | Create native and cross-platform mobile applications for iOS and Android |

### Cloud & Platform

| name_en | name_fr | description_en |
|---|---|---|
| Cloud Architecture – AWS | Architecture infonuagique – AWS | Design and govern workloads on Amazon Web Services, including networking, IAM, and cost management |
| Cloud Architecture – Azure | Architecture infonuagique – Azure | Design and govern workloads on Microsoft Azure including Entra ID and hybrid connectivity |
| Cloud Architecture – GCP | Architecture infonuagique – GCP | Design and govern workloads on Google Cloud Platform |
| Cloud Administration | Administration infonuagique | Provision, monitor, and maintain cloud resources across major public cloud providers |
| Container Technologies | Technologies de conteneurs | Package and run applications using Docker; manage images and container registries |
| Container Orchestration – Kubernetes | Orchestration de conteneurs – Kubernetes | Deploy, scale, and operate containerised workloads on Kubernetes clusters |
| Networking & Infrastructure | Réseautique et infrastructure | Design and administer IP networks, DNS, load balancers, firewalls, and VPNs |
| Platform Engineering | Ingénierie de plateforme | Build and operate internal developer platforms that abstract infrastructure complexity |

### Enterprise Architecture

| name_en | name_fr | description_en |
|---|---|---|
| Enterprise Architecture | Architecture d'entreprise | Align technology strategy with organisational goals using frameworks such as TOGAF |
| Solution Architecture | Architecture de solutions | Design end-to-end technical solutions that meet functional and non-functional requirements |
| Integration Architecture | Architecture d'intégration | Design message-based, event-driven, and API integration patterns across systems |
| Legacy Modernisation | Modernisation des systèmes patrimoniaux | Assess and migrate legacy applications to modern platforms while managing business continuity |

---

## New Domain: `DataAndAnalytics`

| name_en | name_fr | description_en |
|---|---|---|
| Data Engineering | Ingénierie des données | Design and build data pipelines, ETL/ELT processes, and data lakes for analytical workloads |
| Data Modelling | Modélisation des données | Design logical and physical data models, dimensional schemas, and ontologies |
| Data Governance | Gouvernance des données | Establish policies, lineage tracking, cataloguing, and quality controls for enterprise data assets |
| Data Quality Management | Gestion de la qualité des données | Profile, measure, and remediate data quality issues across systems and pipelines |
| Business Intelligence & Reporting | Intelligence d'affaires et rapports | Build dashboards, reports, and self-service analytics using tools such as Power BI or Tableau |
| Data Visualisation | Visualisation des données | Design clear, accurate, and compelling visual representations of complex datasets |
| Statistical Analysis | Analyse statistique | Apply inferential statistics, hypothesis testing, and regression modelling to operational questions |
| Machine Learning Engineering | Ingénierie de l'apprentissage automatique | Train, evaluate, deploy, and monitor supervised and unsupervised ML models in production |
| AI / LLM Integration | Intégration IA / grands modèles de langage | Integrate large language models and generative AI capabilities into products and workflows |
| MLOps | MLOps | Operationalise machine learning workflows including versioning, CI/CD for models, and drift monitoring |
| Geospatial Analysis | Analyse géospatiale | Analyse and visualise geographic and location data using GIS tools and spatial databases |
| Operations Research | Recherche opérationnelle | Apply optimisation, simulation, and decision-analysis methods to operational planning problems |

---

## New Domain: `CyberSecurity`

| name_en | name_fr | description_en |
|---|---|---|
| Security Engineering | Ingénierie de la sécurité | Embed security controls into software and infrastructure design throughout the SDLC |
| Threat Modelling | Modélisation des menaces | Systematically identify and prioritise attack vectors using frameworks such as STRIDE or PASTA |
| Penetration Testing | Test de pénétration | Conduct authorised offensive security assessments to identify exploitable vulnerabilities |
| Vulnerability Management | Gestion des vulnérabilités | Continuously scan, triage, track, and remediate known vulnerabilities across the estate |
| Identity & Access Management | Gestion des identités et des accès | Design and operate IAM systems including SSO, MFA, PAM, and zero-trust access policies |
| Security Operations | Opérations de sécurité | Monitor, detect, investigate, and respond to security events using SIEM and SOAR tooling |
| Incident Response | Réponse aux incidents | Lead the containment, eradication, and recovery from cybersecurity incidents |
| Governance, Risk & Compliance | Gouvernance, risque et conformité | Manage information security risk frameworks, audit readiness, and regulatory compliance |
| Secure Cloud Configuration | Configuration infonuagique sécurisée | Apply cloud security baselines, benchmark controls, and posture management across cloud estates |
| Cryptography & PKI | Cryptographie et ICP | Implement and manage cryptographic controls, certificate authorities, and key management systems |

---

## New Domain: `ProductManagement`

| name_en | name_fr | description_en |
|---|---|---|
| Product Strategy | Stratégie produit | Define product vision, goals, and market positioning aligned to organisational and user needs |
| Backlog Management | Gestion du carnet de travail | Prioritise, refine, and communicate a product backlog that delivers user and business value |
| Product Roadmapping | Feuille de route produit | Develop, maintain, and communicate outcome-oriented product roadmaps to diverse stakeholders |
| Stakeholder Engagement | Engagement des parties prenantes | Identify, map, and manage relationships with stakeholders across delivery, policy, and leadership |
| OKR & Outcome Measurement | Objectifs, résultats clés et mesure des résultats | Define and track Objectives and Key Results to guide teams towards measurable outcomes |
| User Story Writing | Rédaction de récits utilisateurs | Write clear, testable user stories and acceptance criteria that guide delivery teams |
| Market & User Research | Recherche marché et utilisateurs | Gather and synthesise market intelligence, user feedback, and competitive analysis |
| Business Case Development | Élaboration du dossier d'analyse de rentabilisation | Build evidence-based business cases including cost-benefit analysis and options appraisal |
| Benefits Realisation | Réalisation des avantages | Track and report on the delivery of planned benefits throughout the product lifecycle |
| Portfolio Management | Gestion de portefeuille | Manage a portfolio of product investments, balancing risk, capacity, and strategic alignment |

---

## New Domain: `UserExperience`

| name_en | name_fr | description_en |
|---|---|---|
| User Research | Recherche utilisateurs | Plan and conduct qualitative and quantitative research to understand user needs and behaviours |
| Usability Testing | Tests d'utilisabilité | Facilitate moderated and unmoderated sessions to evaluate product usability and identify pain points |
| Service Design | Conception de services | Map and redesign end-to-end service journeys across digital and non-digital touchpoints |
| Interaction Design | Conception d'interaction | Design intuitive, task-centred interfaces that meet user mental models and expectations |
| Information Architecture | Architecture de l'information | Organise and label content and navigation structures to support user findability and comprehension |
| Prototyping & Wireframing | Prototypage et maquettage | Create low- and high-fidelity prototypes to communicate and test design concepts rapidly |
| Design Systems | Systèmes de conception | Build and maintain component libraries and design tokens that ensure consistency at scale |
| Accessibility & Inclusive Design | Accessibilité et conception inclusive | Ensure digital products meet WCAG 2.1 AA standards and serve users with diverse abilities |
| Content Design | Conception de contenu | Write and structure interface copy, notifications, and error messages that guide users clearly |
| UX Strategy | Stratégie UX | Align UX maturity, team structure, and research programmes with product and organisational strategy |

---

## New Domain: `DevOpsAndPlatform`

| name_en | name_fr | description_en |
|---|---|---|
| CI/CD Pipeline Engineering | Ingénierie de pipelines IC/DC | Design, build, and maintain automated build, test, and deployment pipelines |
| Infrastructure as Code | Infrastructure en tant que code | Manage cloud and on-premises infrastructure declaratively using tools such as Terraform or Pulumi |
| GitOps | GitOps | Apply Git-based operational workflows for managing infrastructure and application deployments |
| Site Reliability Engineering | Ingénierie de la fiabilité des sites | Define SLOs and error budgets; build automation to improve service reliability and reduce toil |
| Monitoring & Observability | Surveillance et observabilité | Instrument systems with metrics, logs, and traces; build dashboards and alerting for operational insight |
| Automated Testing | Tests automatisés | Design and implement unit, integration, contract, and end-to-end test suites as part of CI |
| Configuration Management | Gestion des configurations | Manage system configuration at scale using tools such as Ansible, Chef, or Puppet |
| Release Engineering | Ingénierie des livraisons | Manage versioning, release branching, change-freeze windows, and deployment coordination |
| Chaos Engineering | Ingénierie du chaos | Design and run controlled failure experiments to validate system resilience in production |
| FinOps | FinOps | Manage and optimise cloud spend through tagging, budgeting, and rightsizing practices |

---

## New Domain: `AgileDelivery`

| name_en | name_fr | description_en |
|---|---|---|
| Scrum | Scrum | Facilitate and operate Scrum ceremonies and artefacts as Scrum Master or team member |
| Kanban | Kanban | Manage flow, visualise work, and apply Kanban principles to improve delivery throughput |
| SAFe (Scaled Agile Framework) | SAFe (cadre agile à grande échelle) | Apply SAFe roles, events, and artefacts to coordinate multiple agile teams in a programme |
| Lean Thinking | Pensée Lean | Apply Lean principles to eliminate waste, reduce cycle time, and improve value delivery |
| Delivery Management | Gestion de la livraison | Track, report, and unblock delivery across one or more agile teams; manage dependencies and risks |
| Programme Management | Gestion de programme | Plan, govern, and report on a group of related projects or releases to achieve strategic objectives |
| Project Management | Gestion de projet | Manage scope, schedule, cost, and quality for defined-scope technology projects |
| Agile Coaching | Coaching agile | Coach teams and leaders to improve agile practices, culture, and continuous improvement habits |
| Retrospective Facilitation | Animation de rétrospectives | Design and facilitate retrospectives that generate actionable, sustained team improvements |
| Risk & Dependency Management | Gestion des risques et des dépendances | Identify, log, escalate, and mitigate risks and cross-team dependencies in delivery programmes |
| Organisational Change Management | Gestion du changement organisationnel | Plan and execute change management strategies to support adoption of new ways of working |
| Benefits & OKR Tracking | Suivi des avantages et des OKR | Maintain programme-level benefits registers and OKR dashboards aligned to strategic priorities |

---

## Summary of Changes to `SkillDomain` Enum

| Action | Domain | Rationale |
|---|---|---|
| Expand | `InformationTechnology` | Add 15 skills across software engineering, cloud/platform, and enterprise architecture |
| Add | `DataAndAnalytics` | 12 skills covering the full data and AI/ML stack |
| Add | `CyberSecurity` | 10 skills for security engineering, operations, and GRC |
| Add | `ProductManagement` | 10 skills for product ownership and portfolio governance |
| Add | `UserExperience` | 10 skills for research, design, and accessibility |
| Add | `DevOpsAndPlatform` | 10 skills for CI/CD, IaC, SRE, and FinOps |
| Add | `AgileDelivery` | 12 skills for Scrum, SAFe, coaching, and programme delivery |

**Total new skills: ~79** (compared to 7 currently in IT domain)

---

## Notes on Implementation

### Enum Change
The `SkillDomain` enum is stored in PostgreSQL as a native enum type (`skill_domain`). Adding
new variants requires an `ALTER TYPE` migration:

```sql
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'DataAndAnalytics';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'CyberSecurity';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'ProductManagement';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'UserExperience';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'DevOpsAndPlatform';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'AgileDelivery';
```

Note: PostgreSQL `ALTER TYPE … ADD VALUE` is not transactional in older versions (< 12).
Wrap in a migration but test in a staging environment first.

### Rust Enum
Add the six new variants to `SkillDomain` in `src/models/skill.rs` and update all `match`
expressions (particularly in `dummy_capability_data.rs` and anywhere `SkillDomain::iter()` is used).

### Seed Data
Add the skills listed above to `src/database_utils/dummy_capability_data.rs`. The function that
selects random domains for person generation should include the new domains with appropriate
weighting (e.g. `DataAndAnalytics: 10%`, `DevOpsAndPlatform: 8%`, `AgileDelivery: 7%`,
`ProductManagement: 6%`, `UserExperience: 5%`, `CyberSecurity: 5%`).

### Bilingual Names
All skills above include `name_en` and `name_fr`. Descriptions in French (`description_fr`)
should be added by a bilingual reviewer before production use — the table above shows only
`description_en` for brevity.
