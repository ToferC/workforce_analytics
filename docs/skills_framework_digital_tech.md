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
DevOps, agile delivery, procurement, vendor management, or the governance and leadership
disciplines needed to run a civilian digital organisation.

A civilian digital and technology organisation supporting defence forces will typically employ:

- Software engineers, platform engineers, and site reliability engineers
- Data scientists, data engineers, and analysts
- Product managers and delivery managers
- UX researchers and service designers
- DevOps and cloud platform specialists
- Agile coaches and Scrum practitioners
- Cyber security specialists
- Enterprise and solution architects
- Procurement officers and contract managers
- Vendor relationship and performance managers
- Executives, people leaders, and organisational coaches
- Governance, policy, privacy, and information management specialists

The recommendations below expand the existing `InformationTechnology` and `Leadership` domains,
expand or supplement the existing `Management`, `HumanResources`, and `Administration` domains,
and introduce **eight new `SkillDomain` values**.

---

## Recommended New `SkillDomain` Values

```rust
// Add to the SkillDomain enum in src/models/skill.rs
DataAndAnalytics,            // Data engineering, data science, ML, BI, governance
CyberSecurity,               // Security engineering, threat modelling, GRC
ProductManagement,           // Product ownership, roadmapping, stakeholder engagement
UserExperience,              // UX research, service design, accessibility
DevOpsAndPlatform,           // CI/CD, IaC, container orchestration, SRE
AgileDelivery,               // Scrum, Kanban, SAFe, delivery management
ProcurementAndContracting,   // Government procurement, contracting, strategic sourcing
VendorManagement,            // Vendor relationships, performance, third-party risk
OrganisationalLeadership,    // People leadership, executive skills, inclusive leadership
Governance,                  // IT governance, policy, privacy, information management, audit
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

## New Domain: `ProcurementAndContracting`

Government procurement in a defence context follows strict frameworks (e.g. Treasury Board
Contracting Policy, PSPC standing offers, TBIPS, ProServices). The skills below cover the
full procurement lifecycle from planning through contract close-out.

| name_en | name_fr | description_en |
|---|---|---|
| Procurement Strategy & Planning | Stratégie et planification des achats | Develop annual procurement plans aligned to programme roadmaps and departmental spending authority |
| Government Contracting Frameworks | Cadres d'approvisionnement gouvernementaux | Apply Treasury Board and PSPC contracting policies, standing offers, and supply arrangements |
| Statement of Work Development | Élaboration de l'énoncé des travaux | Write clear, measurable statements of work and evaluation criteria for competitive solicitations |
| RFx Development | Élaboration de demandes de soumissions | Prepare Requests for Proposal, Quotation, and Information that comply with competitive procurement rules |
| Bid Evaluation & Selection | Évaluation des soumissions et sélection | Lead technical and financial evaluation of bids against defined criteria; document selection rationale |
| Contract Negotiation | Négociation de contrats | Negotiate commercial terms, pricing, deliverables, and risk allocation with prospective suppliers |
| Contract Administration | Administration des contrats | Manage active contracts including amendments, invoicing, deliverable acceptance, and dispute resolution |
| Strategic Sourcing | Approvisionnement stratégique | Analyse market supply, total cost of ownership, and consolidation opportunities to optimise sourcing decisions |
| Software Licensing & Asset Management | Gestion des licences logicielles et des actifs | Track, rationalise, and renew software licences; manage enterprise agreements with major vendors |
| Procurement Analytics | Analytique des achats | Analyse spend data to identify savings opportunities, compliance gaps, and supplier concentration risk |
| Indigenous & Diversity Procurement | Approvisionnement autochtone et diversifié | Apply mandatory set-asides, Procurement Strategy for Indigenous Business, and supplier diversity targets |
| Procurement Compliance & Audit | Conformité et vérification des achats | Ensure procurement activities meet departmental policy, ATIP obligations, and internal audit standards |

---

## New Domain: `VendorManagement`

| name_en | name_fr | description_en |
|---|---|---|
| Vendor Relationship Management | Gestion des relations avec les fournisseurs | Build and maintain productive working relationships with strategic technology and service suppliers |
| SLA & KPI Management | Gestion des niveaux de service et des ICP | Define, monitor, and enforce service level agreements and key performance indicators with vendors |
| Vendor Performance Reviews | Revues de performance des fournisseurs | Conduct regular structured reviews of vendor delivery against contractual commitments |
| Third-Party Risk Management | Gestion des risques liés aux tiers | Assess, monitor, and remediate security, operational, and financial risks introduced by suppliers |
| Vendor Onboarding & Offboarding | Intégration et départ des fournisseurs | Manage the end-to-end process for bringing new vendors onto and off the organisation's technology estate |
| Escrow & Continuity Planning | Planification de l'entiercement et de la continuité | Establish software escrow arrangements and transition plans to reduce lock-in and ensure continuity |
| Multi-Vendor Coordination | Coordination multi-fournisseurs | Manage interdependencies and integration obligations across multiple concurrent vendor engagements |
| Commercial & Pricing Analysis | Analyse commerciale et tarifaire | Benchmark pricing, analyse commercial proposals, and identify opportunities to reduce unit costs |
| Vendor Consolidation & Rationalisation | Consolidation et rationalisation des fournisseurs | Assess the vendor landscape and consolidate suppliers to reduce overhead and improve leverage |
| IT Asset & License Lifecycle Management | Gestion du cycle de vie des actifs et des licences TI | Track hardware and software assets from procurement through retirement and disposal |

---

## New Domain: `OrganisationalLeadership`

The existing `Leadership` domain captures military leadership skills (Vision Setting, Foresight,
Political Influence, etc.). This new domain addresses the distinct people-leadership, culture,
and executive capabilities needed to run a civilian digital organisation.

| name_en | name_fr | description_en |
|---|---|---|
| People Leadership | Leadership des personnes | Inspire, develop, and retain talent; build team cohesion and psychological safety |
| Inclusive Leadership | Leadership inclusif | Lead in ways that actively value diversity, remove systemic barriers, and ensure equitable outcomes |
| Executive Communication | Communication exécutive | Communicate strategy, decisions, and complex technical concepts clearly to senior and political audiences |
| Strategic Thinking | Réflexion stratégique | Synthesise complex information to identify long-term opportunities and translate vision into direction |
| Managing Through Ambiguity | Gestion dans l'ambiguïté | Maintain team focus and productivity when operating in uncertain, fast-changing environments |
| Talent Development & Coaching | Développement des talents et coaching | Grow individual capabilities through coaching, mentoring, structured feedback, and career conversations |
| Building High-Performing Teams | Constitution d'équipes performantes | Recruit, structure, and develop teams that deliver consistently and improve over time |
| Conflict Resolution | Résolution des conflits | Mediate disagreements constructively and build consensus across competing interests |
| Negotiation | Négociation | Reach workable agreements with peers, partners, and senior stakeholders through principled negotiation |
| Employee Wellbeing & Mental Health | Bien-être des employés et santé mentale | Recognise stress indicators, apply wellness frameworks, and create conditions for sustainable performance |
| Digital Leadership | Leadership numérique | Champion digital transformation, model modern ways of working, and build digital confidence in the organisation |
| Organisational Design | Conception organisationnelle | Design reporting structures, spans of control, and team topologies that enable effective delivery |
| Succession Planning | Planification de la relève | Identify critical roles and develop pipelines of talent to ensure leadership continuity |

---

## New Domain: `Governance`

This domain covers the formal governance, policy, privacy, information management, and audit
capabilities that a civilian organisation operating in a government or defence context must maintain.

### IT & Digital Governance

| name_en | name_fr | description_en |
|---|---|---|
| IT Governance (COBIT / ITIL) | Gouvernance des TI (COBIT / ITIL) | Apply COBIT or ITIL frameworks to align IT decisions with organisational strategy and risk appetite |
| Digital Policy Development | Élaboration de politiques numériques | Research, draft, and consult on policies governing the use of digital tools, data, and AI within the organisation |
| Enterprise Risk Management | Gestion intégrée des risques | Identify, assess, and report on enterprise risks using a structured framework aligned to TB policy |
| IT Audit & Assurance | Vérification et assurance des TI | Plan and execute IT audits; assess controls, produce findings, and track management action plans |
| Business Continuity Planning | Planification de la continuité des activités | Develop and test business continuity and disaster recovery plans for critical digital services |
| Change Management & Change Advisory | Gestion des changements et comité consultatif | Operate a change advisory board process to govern risk to live services from planned changes |

### Privacy & Information Management

| name_en | name_fr | description_en |
|---|---|---|
| Privacy & Access to Information (ATIP) | Protection des renseignements personnels et AIPRP | Process and respond to Access to Information and Privacy requests under the Privacy Act and ATIP legislation |
| Privacy Impact Assessment | Évaluation des facteurs relatifs à la vie privée | Conduct PIAs for new programmes, tools, and data uses; identify and mitigate privacy risks |
| Information Management | Gestion de l'information | Apply government IM frameworks for creation, capture, classification, retention, and disposal of records |
| Records Management | Gestion des documents | Manage official records in compliance with the Library and Archives Canada Act and departmental IM policy |
| Security Classification Management | Gestion de la classification de sécurité | Apply government security classification schemes (Protected A/B/C, Secret, Top Secret) to information assets |
| Open Government & Transparency | Gouvernement ouvert et transparence | Manage proactive disclosure obligations, open data publishing, and departmental transparency commitments |

### Corporate Governance

| name_en | name_fr | description_en |
|---|---|---|
| Corporate Planning & Reporting | Planification et rapports organisationnels | Lead departmental planning cycles (DPR, DP, MAF) and produce results-based performance reports |
| Treasury Board Submissions | Présentations au Conseil du Trésor | Develop TB submissions, Memoranda to Cabinet, and other central-agency approval documents |
| Internal Audit | Vérification interne | Plan and execute internal audits against the TB Policy on Internal Audit; report to the audit committee |
| Evaluation | Évaluation | Design and conduct programme evaluations aligned to the TB Policy on Results |
| Legal & Policy Interpretation | Interprétation juridique et des politiques | Interpret legislation, regulations, and policy instruments to advise programme and delivery teams |
| Official Languages Management | Gestion des langues officielles | Ensure compliance with the Official Languages Act in service delivery, staffing, and workplace communications |

---

## Expanded Domain Notes: Existing Domains

### `Leadership` (existing — recommended expansions)
The current seed skills (Vision Setting, Innovation, Foresight, Political Influence, Mobilizing People)
remain valid for senior leadership. Consider adding:

| name_en | name_fr | description_en |
|---|---|---|
| Servant Leadership | Leadership serviteur | Prioritise the needs of the team; remove blockers and enable others to do their best work |
| Systems Thinking | Pensée systémique | Analyse interdependencies across complex sociotechnical systems to understand second-order effects |
| Psychological Safety | Sécurité psychologique | Create conditions where team members feel safe to speak up, experiment, and learn from failure |

### `Management` (existing — recommended expansions)
Current skills: People Management, Action Management, Financial Management, Performance Management. Add:

| name_en | name_fr | description_en |
|---|---|---|
| Workforce Planning | Planification des effectifs | Forecast and plan for the talent, skills, and headcount needed to meet future delivery commitments |
| Resource & Capacity Management | Gestion des ressources et de la capacité | Allocate people, budget, and tooling across competing priorities to maximise delivery throughput |
| Knowledge Management | Gestion des connaissances | Design systems to capture, share, and reuse institutional knowledge across the organisation |

### `HumanResources` (existing — recommended expansions)
Current skills: Staffing, Classification, Recruiting, Pay and Compensation. Add:

| name_en | name_fr | description_en |
|---|---|---|
| Learning & Development | Apprentissage et perfectionnement | Design and manage programmes that build the skills and capabilities the organisation needs over time |
| Talent Management | Gestion des talents | Identify high-potential employees, manage succession pipelines, and reduce attrition risk |
| Equity, Diversity & Inclusion | Équité, diversité et inclusion | Design and implement EDI strategies that address systemic barriers and improve workforce representation |
| Employee Relations | Relations avec les employés | Manage labour relations, grievances, and collective agreement obligations in a unionised environment |
| Occupational Health & Safety | Santé et sécurité au travail | Administer OHS obligations, workplace accommodation, and return-to-work programmes |

---

## Summary of Changes to `SkillDomain` Enum

| Action | Domain | Skills | Rationale |
|---|---|---|---|
| Expand | `InformationTechnology` | ~22 | Software engineering, cloud/platform, and enterprise architecture |
| Add | `DataAndAnalytics` | 12 | Full data and AI/ML stack |
| Add | `CyberSecurity` | 10 | Security engineering, operations, and GRC |
| Add | `ProductManagement` | 10 | Product ownership and portfolio governance |
| Add | `UserExperience` | 10 | Research, design, and accessibility |
| Add | `DevOpsAndPlatform` | 10 | CI/CD, IaC, SRE, and FinOps |
| Add | `AgileDelivery` | 12 | Scrum, SAFe, coaching, and programme delivery |
| Add | `ProcurementAndContracting` | 12 | Government procurement lifecycle, strategic sourcing, compliance |
| Add | `VendorManagement` | 10 | Vendor relationships, SLAs, third-party risk, commercial analysis |
| Add | `OrganisationalLeadership` | 13 | People leadership, executive communication, inclusive leadership |
| Add | `Governance` | 15 | IT governance, privacy/ATIP, information management, corporate governance |
| Expand | `Leadership` | +3 | Servant leadership, systems thinking, psychological safety |
| Expand | `Management` | +3 | Workforce planning, capacity management, knowledge management |
| Expand | `HumanResources` | +5 | L&D, talent management, EDI, employee relations, OHS |

**Total new / updated skills: ~147** across 14 domains (compared to the original 7 IT-only skills)

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
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'ProcurementAndContracting';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'VendorManagement';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'OrganisationalLeadership';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'Governance';
```

Note: PostgreSQL `ALTER TYPE … ADD VALUE` is not transactional in older versions (< 12).
Wrap in a migration but test in a staging environment first.

### Rust Enum
Add the six new variants to `SkillDomain` in `src/models/skill.rs` and update all `match`
expressions (particularly in `dummy_capability_data.rs` and anywhere `SkillDomain::iter()` is used).

### Seed Data
Add the skills listed above to `src/database_utils/dummy_capability_data.rs`. The function that
selects random domains for person generation should include the new domains with appropriate
weighting — for example:

| Domain | Suggested weight |
|---|---|
| `DataAndAnalytics` | 10% |
| `DevOpsAndPlatform` | 8% |
| `AgileDelivery` | 7% |
| `ProductManagement` | 6% |
| `UserExperience` | 5% |
| `CyberSecurity` | 5% |
| `ProcurementAndContracting` | 4% |
| `VendorManagement` | 3% |
| `OrganisationalLeadership` | 5% |
| `Governance` | 4% |

Roles such as executives, directors, and branch heads should skew toward
`OrganisationalLeadership` and `Governance`; procurement officers toward
`ProcurementAndContracting` and `VendorManagement`.

### Bilingual Names
All skills above include `name_en` and `name_fr`. Descriptions in French (`description_fr`)
should be added by a bilingual reviewer before production use — the table above shows only
`description_en` for brevity.
