# Consolidated Skills Framework
## Defence Digital & Technology Organisation — Combined Military & Civilian

**Version**: 1.0 — Consolidated review draft  
**Replaces**: `skills_framework_digital_tech.md` (additive draft)  
**Total**: 17 domains · 197 skills

---

## Consolidation Principles

Every skill appears in exactly one domain. The following rules resolved conflicts:

| Rule | Example resolution |
|---|---|
| A skill belongs where its **primary practitioner** sits, not where it is used | _Data Analysis_ removed from Intelligence; lives in Data, Analytics & AI |
| Skills that are **specialisations of a broader skill** in the same domain are merged | _SLA Management_ + _Vendor Performance Reviews_ → single skill |
| Skills duplicated across multiple proposed new domains are **cut to one home** | _Benefits & OKR Tracking_ (AgileDelivery) removed; _Benefits Realisation_ (ProductManagement) is the keeper |
| Organisational support domains (Finance, HR, Comms, Admin) **merged** into Corporate Services to reduce domain count | Four small domains → one domain, 17 skills |
| Leadership and Management **merged** into a single domain; overlap with HR strategic skills resolved in favour of this domain | _Leadership_ + _Management_ + _OrganisationalLeadership_ → _People & Organisational Leadership_ |
| Military-specific domains are **left intact**; only cross-domain duplicates removed from them | _Intelligence Analysis_ removed from Strategy (its home is Intelligence); _Data Analysis_ removed from Intelligence (its home is Data, Analytics & AI) |

---

## Domain Overview

| # | Domain (Rust enum) | Plain name | Primary population |
|---|---|---|---|
| 1 | `Combat` | Combat Operations | Military operators |
| 2 | `Intelligence` | Intelligence & Reconnaissance | Military/civilian intelligence analysts |
| 3 | `Strategy` | Strategy & Operational Planning | Military planners; senior executives |
| 4 | `Engineering` | Military Engineering | Combat engineers; EOD |
| 5 | `Medical` | Medical & Health Services | Military/civilian medical staff |
| 6 | `JointOperations` | Joint & Coalition Operations | Joint and multinational force personnel |
| 7 | `SoftwareEngineering` | Software Engineering | Developers (all languages/stacks) |
| 8 | `CloudPlatformDevOps` | Cloud, Platform & DevOps | Infrastructure, platform, SRE, architects |
| 9 | `DataAnalyticsAndAI` | Data, Analytics & AI | Data engineers, scientists, analysts, ML |
| 10 | `CyberSecurity` | Cyber Security | Security engineers, analysts, GRC |
| 11 | `ProductManagement` | Product Management | Product managers, product owners |
| 12 | `AgileAndDelivery` | Agile & Delivery | Delivery managers, Scrum Masters, programme managers |
| 13 | `UserExperience` | User Experience & Design | UX researchers, service designers, content designers |
| 14 | `ProcurementAndVendorManagement` | Procurement & Vendor Management | Procurement officers, commercial managers |
| 15 | `PeopleAndOrganisationalLeadership` | People & Organisational Leadership | Leaders, managers, HR strategic, coaches |
| 16 | `Governance` | Governance & Compliance | Policy, audit, privacy, IM, GRC analysts |
| 17 | `CorporateServices` | Corporate Services | Finance, HR operational, communications, admin |

> **Enums renamed from existing codebase**: `Engineering` retains its Rust name but its scope is explicitly military. `InformationTechnology` is **retired** and split into `SoftwareEngineering` and `CloudPlatformDevOps`. `Leadership`, `Management`, `HumanResources`, `Finance`, `Communications`, and `Administration` are **retired** and their skills redistributed; see implementation notes.

---

## 1 · Combat Operations

*Direct combat skills and individual military operator tasks.*

| name_en | name_fr | description_en |
|---|---|---|
| Infantry Operations | Opérations d'infanterie | Lead and execute ground assault, defensive, and patrol tasks as an infantry soldier |
| Armoured Operations | Opérations blindées | Operate and employ armoured fighting vehicles in combined arms operations |
| Artillery Operations | Opérations d'artillerie | Deliver indirect fire support using artillery and mortar systems |
| Close Quarters Combat | Combat rapproché | Engage enemy forces in confined spaces using close-range weapon techniques |
| Marksmanship | Tir de précision | Accurately engage targets with individual and crew-served weapons systems |
| Small Unit Tactics | Tactiques de petites unités | Lead fire-and-movement, patrolling, and other small unit battle tasks |
| Battlefield Awareness | Conscience situationnelle au combat | Understand and communicate the tactical picture; apply ISTAR in a ground combat environment |
| Weapons Systems Employment | Emploi des systèmes d'armes | Operate, maintain, and employ weapons platforms and munitions in accordance with ROE |
| Urban Operations | Opérations en milieu urbain | Plan and execute offensive, defensive, and stability tasks in built-up areas |

---

## 2 · Intelligence & Reconnaissance

*Collection, processing, and reporting of military and operational intelligence.*  
**Note**: Analytical tradecraft using statistical or data tools belongs in domain 9 (Data, Analytics & AI), not here.

| name_en | name_fr | description_en |
|---|---|---|
| Intelligence Collection Planning | Planification de la collecte du renseignement | Plan and task collection activities across human, technical, and open-source disciplines |
| Reconnaissance | Reconnaissance | Conduct ground and aerial reconnaissance to acquire information on terrain, enemy, and civil factors |
| Surveillance | Surveillance | Monitor areas, routes, and points of interest using ground sensors, UAVs, and ISR assets |
| Signals Intelligence | Renseignement d'origine électromagnétique | Collect, process, and exploit adversary signals to identify communications and electronic activity |
| Human Intelligence | Renseignement d'origine humaine | Build and manage human source networks; conduct source handling within legal and ethical constraints |
| Threat Assessment | Évaluation des menaces | Synthesise multi-source intelligence to assess adversary intent, capability, and likely courses of action |
| All-Source Intelligence Analysis | Analyse de renseignement toutes sources | Integrate products from multiple collection disciplines to produce coherent, actionable assessments |
| Intelligence Reporting | Rapports de renseignement | Produce timely, accurate, classified intelligence products tailored to operational commander needs |

---

## 3 · Strategy & Operational Planning

*Military operational planning and high-level organisational strategy.*  
**Note**: Intelligence Analysis removed — it belongs in domain 2.

| name_en | name_fr | description_en |
|---|---|---|
| Strategic Planning | Planification stratégique | Develop long-range plans that align organisational capability, resources, and objectives |
| Military Doctrine | Doctrine militaire | Apply, develop, and update doctrinal publications to guide military force employment |
| Operational Planning | Planification opérationnelle | Apply the Military Decision-Making Process or equivalent to plan joint and combined operations |
| Tactical Analysis | Analyse tactique | Analyse terrain, weather, and adversary factors to inform course-of-action development |
| Force Structure Planning | Planification de la structure des forces | Assess and design force structures to meet operational requirements within resource constraints |
| Mission Analysis | Analyse de la mission | Deconstruct commander's intent; identify specified, implied, and essential tasks |
| Campaign Planning | Planification de la campagne | Design sequenced military campaigns that achieve strategic objectives through a series of operations |
| Organisational Strategy | Stratégie organisationnelle | Define the long-range direction, priorities, and measurable goals of a civilian or defence support organisation |

---

## 4 · Military Engineering

*Combat and technical engineering in support of military operations.*

| name_en | name_fr | description_en |
|---|---|---|
| Explosive Ordnance Disposal | Neutralisation des engins explosifs | Locate, identify, render safe, and dispose of IEDs and conventional munitions |
| Combat Engineering | Génie de combat | Execute breaching, bridging, obstacle emplacement, and mobility/counter-mobility tasks |
| Fortification & Field Defences | Fortification et défenses de campagne | Design and construct field defences, strong points, and survivability positions |
| Military Bridge Construction | Construction de ponts militaires | Plan and erect military bridging to support force crossing of obstacles |
| Demolitions | Démolitions | Apply demolition techniques for route clearance, infrastructure denial, and obstacle creation |
| Mine Warfare | Guerre des mines | Lay, record, and clear minefields in compliance with operational plans and Law of Armed Conflict |
| Technical Intelligence | Renseignement technique | Analyse captured enemy equipment and materiel to derive intelligence value and capability assessments |

---

## 5 · Medical & Health Services

*Medical support to military and civilian personnel in field and garrison settings.*

| name_en | name_fr | description_en |
|---|---|---|
| Combat Medicine | Médecine de combat | Provide immediate trauma care on the battlefield using Tactical Combat Casualty Care protocols |
| Battlefield Trauma Care | Soins aux traumatisés de combat | Manage traumatic injuries including haemorrhage control, airway management, and burns in austere environments |
| Field Surgery | Chirurgie de campagne | Perform damage-control surgical interventions in deployed medical facilities |
| Emergency Medicine | Médecine d'urgence | Assess and stabilise acute medical and surgical emergencies in field and garrison settings |
| Triage | Triage | Categorise casualties by severity and survivability to prioritise scarce medical resources |
| Medical Evacuation | Évacuation médicale | Coordinate and execute ground and aeromedical evacuation of casualties to definitive care |
| CBRN Medical Response | Réponse médicale CBRN | Diagnose and treat casualties exposed to chemical, biological, radiological, and nuclear agents |

---

## 6 · Joint & Coalition Operations

*Coordination across military services, allied nations, and partner organisations.*

| name_en | name_fr | description_en |
|---|---|---|
| Joint Operations | Opérations interarmées | Plan and execute operations integrating land, maritime, air, space, and cyber effects |
| Coalition Warfare | Guerre de coalition | Operate within multinational command structures while managing national caveats and interoperability constraints |
| Multinational Staff Work | Travail d'état-major multinational | Conduct staff duties in combined or coalition headquarters with personnel from multiple nations |
| Military Interoperability | Interopérabilité militaire | Apply shared doctrine, procedures, and standards (e.g. STANAG) to enable combined operations |
| Combined Arms Integration | Intégration des armes combinées | Synchronise effects from multiple combat arms and enabling functions to achieve tactical objectives |
| Liaison & Coordination | Liaison et coordination | Represent a formation at external headquarters; maintain information flow across organisational boundaries |
| International Military Cooperation | Coopération militaire internationale | Build and sustain bilateral and multilateral defence relationships including capacity building and training |

---

## 7 · Software Engineering

*Design, build, test, and maintain software applications and services.*

| name_en | name_fr | description_en |
|---|---|---|
| Software Engineering – Python | Génie logiciel – Python | Design and build production-quality Python services and applications including testing and packaging |
| Software Engineering – TypeScript / JavaScript | Génie logiciel – TypeScript / JavaScript | Develop front-end and server-side applications using modern JavaScript and TypeScript ecosystems |
| Software Engineering – Rust | Génie logiciel – Rust | Build high-performance, memory-safe systems-level software in Rust |
| Software Engineering – Go | Génie logiciel – Go | Write concurrent, cloud-native services and CLI tooling in Go |
| Software Engineering – Java / Kotlin | Génie logiciel – Java / Kotlin | Develop enterprise back-end services and Android applications using JVM languages |
| Front-end Development | Développement frontal | Build responsive, accessible web UIs using component frameworks such as React or Vue |
| Back-end Development | Développement dorsal | Implement server-side business logic, data access layers, and service orchestration |
| Mobile Development | Développement mobile | Create native and cross-platform mobile applications for iOS and Android |
| API Design | Conception d'API | Design consistent, versioned, and secure REST, GraphQL, and gRPC APIs |
| Microservices & Distributed Systems | Microservices et systèmes distribués | Design and operate independently deployable services; handle distributed systems concerns (consistency, resilience) |
| Database Engineering | Ingénierie des bases de données | Model, implement, and optimise relational and document databases; manage schema migrations and query tuning |

---

## 8 · Cloud, Platform & DevOps

*Infrastructure, platform, deployment pipeline, and operational reliability of technology systems.*  
Includes enterprise architecture — architects work most closely with this professional family.

| name_en | name_fr | description_en |
|---|---|---|
| **Cloud** | | |
| Cloud Architecture – AWS | Architecture infonuagique – AWS | Design and govern workloads on Amazon Web Services including networking, IAM, and multi-account strategy |
| Cloud Architecture – Azure | Architecture infonuagique – Azure | Design and govern workloads on Microsoft Azure including Entra ID, hybrid connectivity, and GC Cloud |
| Cloud Architecture – GCP | Architecture infonuagique – GCP | Design and govern workloads on Google Cloud Platform |
| Cloud Administration | Administration infonuagique | Provision, monitor, and maintain cloud resources; manage accounts, spend, and operational hygiene |
| Container Technologies | Technologies de conteneurs | Package and run applications using Docker; manage images, registries, and multi-stage builds |
| Container Orchestration – Kubernetes | Orchestration – Kubernetes | Deploy, scale, and secure containerised workloads on Kubernetes; manage cluster lifecycle |
| Networking & Infrastructure | Réseautique et infrastructure | Design and administer IP networks, DNS, load balancers, firewalls, and zero-trust networking |
| Platform Engineering | Ingénierie de plateforme | Build internal developer platforms (IDPs) that abstract infrastructure and accelerate delivery teams |
| **DevOps & Reliability** | | |
| CI/CD Pipeline Engineering | Ingénierie de pipelines IC/DC | Design, build, and maintain automated build, test, security-scan, and deployment pipelines |
| Infrastructure as Code | Infrastructure en tant que code | Manage cloud and on-premises infrastructure declaratively using Terraform, Pulumi, or Bicep |
| Site Reliability Engineering | Ingénierie de la fiabilité des sites | Define SLOs and error budgets; build toil-reduction automation and runbooks for production services |
| Monitoring & Observability | Surveillance et observabilité | Instrument systems with metrics, structured logs, and traces; build alerting and operational dashboards |
| Configuration Management | Gestion des configurations | Manage system configuration at scale using Ansible or equivalent; enforce desired state across fleets |
| Release Engineering | Ingénierie des livraisons | Govern versioning strategies, change-freeze windows, deployment coordination, and rollback plans |
| FinOps | FinOps | Manage and optimise cloud spend through tagging, budgets, rightsizing, and commitment strategies |
| **Architecture** | | |
| Enterprise Architecture | Architecture d'entreprise | Align technology portfolio, standards, and investment with organisational strategy (e.g. TOGAF) |
| Solution Architecture | Architecture de solutions | Design end-to-end technical solutions balancing functional requirements, cost, security, and operability |
| Integration Architecture | Architecture d'intégration | Design event-driven, message-based, and API integration patterns across distributed systems |
| Legacy Modernisation | Modernisation des systèmes patrimoniaux | Assess and migrate legacy applications to modern platforms while managing business continuity |

---

## 9 · Data, Analytics & AI

*Engineering, analysis, and operationalisation of data and AI/ML capabilities.*  
**Note**: This is the correct home for analytical tradecraft (statistical modelling, ML) even when applied to military intelligence questions.

| name_en | name_fr | description_en |
|---|---|---|
| Data Engineering | Ingénierie des données | Build and maintain data pipelines, ETL/ELT processes, streaming architectures, and data lakes |
| Data Modelling | Modélisation des données | Design logical and physical data models, dimensional schemas, and ontologies for analytical workloads |
| Data Governance | Gouvernance des données | Establish policies, cataloguing, lineage tracking, and quality controls for enterprise data assets |
| Data Quality Management | Gestion de la qualité des données | Profile, measure, and remediate data quality issues; define and monitor quality metrics |
| Business Intelligence & Reporting | Intelligence d'affaires et rapports | Build self-service dashboards and scheduled reports using tools such as Power BI, Tableau, or Looker |
| Data Visualisation | Visualisation des données | Design clear, accurate visual representations of complex datasets for decision-maker audiences |
| Statistical Analysis | Analyse statistique | Apply inferential statistics, hypothesis testing, and regression modelling to operational and research questions |
| Machine Learning Engineering | Ingénierie de l'apprentissage automatique | Train, evaluate, deploy, and monitor supervised and unsupervised ML models at production scale |
| AI / LLM Integration | Intégration IA / grands modèles de langage | Integrate large language models and generative AI into products; apply RAG, prompt engineering, and fine-tuning |
| MLOps | MLOps | Operationalise ML workflows: model versioning, CI/CD for models, feature stores, and drift monitoring |
| Geospatial Analysis | Analyse géospatiale | Analyse and visualise geographic and location data using GIS tools, spatial databases, and remote sensing |
| Operations Research | Recherche opérationnelle | Apply optimisation, simulation, and decision modelling to force planning, logistics, and scheduling problems |

---

## 10 · Cyber Security

*Protection of information systems, data, and infrastructure from cyber threats.*  
**Note**: `Information Security Governance` here covers information security frameworks (ISO 27001, NIST CSF, ITSG-33) and is **distinct** from enterprise governance in domain 16.

| name_en | name_fr | description_en |
|---|---|---|
| Security Engineering | Ingénierie de la sécurité | Embed security controls into software and infrastructure design throughout the development lifecycle |
| Threat Modelling | Modélisation des menaces | Identify and prioritise attack vectors using frameworks such as STRIDE, PASTA, or MITRE ATT&CK |
| Penetration Testing | Test de pénétration | Conduct authorised offensive security assessments to identify and demonstrate exploitable vulnerabilities |
| Vulnerability Management | Gestion des vulnérabilités | Scan, triage, prioritise, and track remediation of known vulnerabilities across the technology estate |
| Identity & Access Management | Gestion des identités et des accès | Design and operate IAM including SSO, MFA, PAM, and zero-trust access control policies |
| Security Operations | Opérations de sécurité | Monitor, detect, investigate, and respond to threats using SIEM, SOAR, and threat intelligence platforms |
| Incident Response | Réponse aux incidents | Lead end-to-end response to cybersecurity incidents: containment, eradication, recovery, and lessons learned |
| Information Security Governance | Gouvernance de la sécurité de l'information | Apply and assess information security frameworks (ISO 27001, NIST CSF); manage security risk and compliance |
| Cloud Security Posture Management | Gestion de la posture de sécurité infonuagique | Apply cloud security baselines (CIS Benchmarks, Well-Architected) and manage cloud security posture continuously |
| Cryptography & PKI | Cryptographie et ICP | Implement and manage cryptographic controls, certificate authorities, and enterprise key management systems |

---

## 11 · Product Management

*Define what to build and why; manage the product lifecycle from discovery through value measurement.*  
**Note**: `Market & Competitive Research` here focuses on market intelligence and competitive analysis; user behavioural research belongs in domain 13 (User Experience).  
**Note**: `Benefits Realisation` is the single home for benefits tracking — removed from AgileDelivery.

| name_en | name_fr | description_en |
|---|---|---|
| Product Strategy | Stratégie produit | Define and communicate product vision, positioning, and long-term direction aligned to user and organisational needs |
| Product Roadmapping | Feuille de route produit | Build and maintain outcome-oriented roadmaps; communicate priorities and trade-offs to diverse stakeholders |
| Backlog Management | Gestion du carnet de travail | Prioritise, groom, and communicate a product backlog that maximises user and business value per sprint |
| User Story Writing | Rédaction de récits utilisateurs | Write well-scoped, testable user stories and acceptance criteria that guide delivery teams unambiguously |
| Market & Competitive Research | Recherche marché et concurrentielle | Gather and synthesise market intelligence, user feedback, and competitor analysis to inform product decisions |
| Stakeholder Engagement | Engagement des parties prenantes | Identify, map, and actively manage relationships with stakeholders across delivery, policy, and senior leadership |
| Business Case Development | Élaboration du dossier d'analyse de rentabilisation | Build evidence-based investment proposals including options analysis, cost-benefit assessment, and risk summary |
| OKR & Outcome Measurement | Mesure des résultats et des OKR | Define and track Objectives and Key Results; ensure teams have clear, measurable, and current success criteria |
| Benefits Realisation | Réalisation des avantages | Track and demonstrate delivery of planned benefits; maintain benefits registers and report to governance bodies |
| Portfolio Prioritisation | Priorisation du portefeuille | Manage and prioritise a portfolio of product investments against organisational strategy and resource constraints |

---

## 12 · Agile & Delivery

*Lead teams in planning, executing, and continuously improving delivery of digital products.*  
**Note**: `Organisational Change Management` removed — it belongs in domain 15 (People & Organisational Leadership).  
**Note**: Benefits and OKR tracking removed — single home is domain 11 (Product Management).

| name_en | name_fr | description_en |
|---|---|---|
| Scrum | Scrum | Facilitate Scrum ceremonies, artefacts, and team norms as Scrum Master or as a team member |
| Kanban | Kanban | Apply flow management, WIP limits, and pull-based scheduling to improve delivery predictability |
| SAFe – Scaled Agile Framework | SAFe – cadre agile à grande échelle | Apply SAFe roles, events, and value streams to coordinate multiple teams at programme scale |
| Lean Thinking | Pensée Lean | Apply Lean principles to eliminate waste, shorten lead times, and create continuous flow |
| Delivery Management | Gestion de la livraison | Track and unblock delivery for one or more agile teams; manage impediments and team-level risks |
| Programme Management | Gestion de programme | Plan, govern, and report on a group of related products or projects to achieve shared strategic outcomes |
| Project Management | Gestion de projet | Manage scope, schedule, budget, and quality for defined-scope technology or change projects |
| Agile Coaching | Coaching agile | Coach teams and leaders toward higher agile maturity; facilitate continuous improvement and self-organisation |
| Retrospective Facilitation | Animation de rétrospectives | Design and facilitate retrospectives that surface real issues and generate sustained, actionable improvements |
| Delivery Risk Management | Gestion des risques de livraison | Identify, log, communicate, and mitigate delivery risks, constraints, and cross-team dependencies |

---

## 13 · User Experience & Design

*Understand user needs and design products, services, and interfaces that meet them.*  
**Note**: `User Research` here focuses on behavioural, attitudinal, and usability research; market and competitive research belongs in domain 11 (Product Management).

| name_en | name_fr | description_en |
|---|---|---|
| User Research | Recherche utilisateurs | Plan and conduct qualitative and quantitative research to understand user needs, mental models, and behaviours |
| Usability Testing | Tests d'utilisabilité | Facilitate moderated and unmoderated sessions to evaluate usability; synthesise findings into clear recommendations |
| Service Design | Conception de services | Map and redesign end-to-end service journeys across digital and non-digital touchpoints; create service blueprints |
| Interaction Design | Conception d'interaction | Design task-centred interface flows, states, and feedback mechanisms that match user expectations |
| Information Architecture | Architecture de l'information | Organise and label content, navigation, and taxonomy to support user findability and task completion |
| Prototyping & Wireframing | Prototypage et maquettage | Create low- and high-fidelity prototypes to test and communicate design concepts before build |
| Design Systems | Systèmes de conception | Build and govern component libraries, design tokens, and usage guidelines to ensure consistent UI at scale |
| Accessibility & Inclusive Design | Accessibilité et conception inclusive | Ensure products meet WCAG 2.1 AA and are usable by people with a diverse range of abilities and contexts |
| Content Design | Conception de contenu | Write and structure interface copy, error messages, and notifications that help users complete tasks |
| UX Strategy | Stratégie UX | Define UX maturity, team investment, research programme, and tooling strategy aligned to product and org goals |

---

## 14 · Procurement & Vendor Management

*Acquire technology goods and services within government policy; manage supplier relationships and performance.*  
**Consolidated from**: `ProcurementAndContracting` + `VendorManagement`.  
**Removed**: Procurement Compliance & Audit (implied by Government Contracting Frameworks + Internal Audit in Governance), Vendor Consolidation (implied by Strategic Sourcing), Escrow Planning (too niche).  
**Merged**: SLA Management + Vendor Performance Reviews → single skill; Software Licensing + IT Asset Lifecycle → single skill.

| name_en | name_fr | description_en |
|---|---|---|
| **Procurement** | | |
| Procurement Strategy & Planning | Stratégie et planification des achats | Develop annual procurement plans aligned to programme roadmaps and departmental spending authorities |
| Government Contracting Frameworks | Cadres d'approvisionnement gouvernementaux | Apply Treasury Board, PSPC, TBIPS, and ProServices policies for competitive and sole-source procurement |
| Statement of Work Development | Élaboration de l'énoncé des travaux | Write measurable statements of work, technical requirements, and evaluation criteria for solicitations |
| RFx Development | Élaboration de demandes de soumissions | Prepare RFP, RFQ, Standing Offer, and Supply Arrangement solicitation documents that comply with competitive rules |
| Bid Evaluation & Selection | Évaluation des soumissions et sélection | Lead technical and financial evaluation of bids; document selection rationale in compliance with procurement policy |
| Contract Negotiation | Négociation de contrats | Negotiate commercial terms, deliverables, pricing, liability, and risk allocation with prospective suppliers |
| Contract Administration | Administration des contrats | Manage active contracts: amendments, deliverable acceptance, invoicing, dispute resolution, and close-out |
| Strategic Sourcing | Approvisionnement stratégique | Analyse the supply market, total cost of ownership, and consolidation opportunities to optimise sourcing decisions |
| Indigenous & Diversity Procurement | Approvisionnement autochtone et diversifié | Apply Procurement Strategy for Indigenous Business mandatory set-asides and supplier diversity commitments |
| Procurement Analytics | Analytique des achats | Analyse spend data to identify savings opportunities, compliance gaps, and supply concentration risks |
| **Vendor Management** | | |
| Vendor Relationship Management | Gestion des relations fournisseurs | Build and maintain productive strategic relationships with key technology and service suppliers |
| SLA & Vendor Performance Management | Gestion des niveaux de service et de la performance fournisseur | Define, monitor, and enforce SLAs; conduct structured performance reviews against contractual commitments |
| Third-Party Risk Management | Gestion des risques liés aux tiers | Assess, monitor, and remediate security, operational, financial, and concentration risks from suppliers |
| Vendor Onboarding & Offboarding | Intégration et départ des fournisseurs | Manage the end-to-end process for engaging and safely exiting vendors from the technology estate |
| Commercial & Pricing Analysis | Analyse commerciale et tarifaire | Benchmark vendor pricing, analyse commercial proposals, and negotiate cost reductions and improved terms |
| Software & Asset Lifecycle Management | Gestion du cycle de vie des logiciels et des actifs | Track hardware, software licences, and enterprise agreements from acquisition through renewal and retirement |
| Multi-Vendor Coordination | Coordination multi-fournisseurs | Manage interdependencies and integration obligations across concurrent vendor engagements |

---

## 15 · People & Organisational Leadership

*Lead and develop people; build organisational capability; manage the human dimensions of a digital organisation.*  
**Consolidated from**: `Leadership` + `Management` + `OrganisationalLeadership`.  
**Removed**: Foresight (redundant with Vision Setting + Strategic Thinking), Action Management (too generic), Financial Management (for Finance professionals, not line managers), Managing Through Ambiguity (implied by other skills), Succession Planning (implied by Workforce Planning + Talent Development).  
**Moved here from AgileDelivery**: Organisational Change Management.

| name_en | name_fr | description_en |
|---|---|---|
| **Direction** | | |
| Vision Setting | Établissement de la vision | Articulate a compelling long-term direction that energises people and guides day-to-day decision-making |
| Strategic Thinking | Réflexion stratégique | Synthesise complex information to identify opportunities, risks, and the second-order implications of choices |
| Innovation | Innovation | Create conditions for novel ideas to emerge; champion experimentation and build tolerance for productive failure |
| Digital Leadership | Leadership numérique | Champion digital transformation; model modern ways of working and build digital confidence across the organisation |
| Political Acuity | Acuité politique | Navigate political environments, build coalitions, and manage upward and lateral influence effectively |
| **Leading People** | | |
| People Leadership | Leadership des personnes | Inspire, develop, and retain talent; create the conditions in which individuals and teams do their best work |
| Inclusive Leadership | Leadership inclusif | Lead in ways that actively value diversity, dismantle barriers, and produce equitable outcomes for all staff |
| Servant Leadership | Leadership serviteur | Prioritise team needs over personal agenda; remove blockers and enable others to perform at their best |
| Mobilizing People | Mobilisation des personnes | Build energy, shared commitment, and collective momentum toward goals through authentic engagement |
| Psychological Safety | Sécurité psychologique | Create environments where people feel safe to speak up, raise concerns, take risks, and learn from mistakes |
| Building High-Performing Teams | Constitution d'équipes performantes | Form, develop, and sustain teams that deliver consistently and improve through structured reflection |
| **Developing People** | | |
| Talent Development & Coaching | Développement des talents et coaching | Grow individuals through structured coaching, mentoring, feedback conversations, and deliberate career development |
| Performance Management | Gestion du rendement | Set clear expectations, provide ongoing feedback, and conduct fair and constructive performance reviews |
| Employee Wellbeing | Bien-être des employés | Recognise stress and burnout; apply wellness approaches and adjust working conditions proactively |
| **Communication & Influence** | | |
| Executive Communication | Communication exécutive | Communicate strategy, decisions, and technical complexity clearly and confidently to senior and political audiences |
| Negotiation | Négociation | Reach durable agreements through principled negotiation, separating positions from underlying interests |
| Conflict Resolution | Résolution des conflits | Mediate interpersonal and inter-team disagreements; restore productive working relationships |
| **Organisational Capability** | | |
| Organisational Design | Conception organisationnelle | Design reporting structures, team topologies, and spans of control that enable effective delivery at scale |
| Workforce Planning | Planification des effectifs | Forecast and plan for the talent and skills needed to meet future delivery commitments and strategic goals |
| Resource & Capacity Management | Gestion des ressources et de la capacité | Allocate people, budget, and tooling across competing priorities to maximise throughput and team sustainability |
| Organisational Change Management | Gestion du changement organisationnel | Plan and embed organisational changes so people are ready, willing, and able to adopt new ways of working |

---

## 16 · Governance & Compliance

*Oversight, policy, audit, privacy, and information management for a government technology organisation.*  
**Note**: `Information Security Governance` lives in domain 10 (Cyber Security) — this domain covers enterprise governance, not infosec frameworks.  
**Note**: IT Change Governance here (Change Advisory Board process) is **distinct** from `Organisational Change Management` in domain 15.

| name_en | name_fr | description_en |
|---|---|---|
| **IT & Digital Governance** | | |
| IT Governance | Gouvernance des TI | Apply COBIT, ITIL, or equivalent frameworks to align IT decisions, controls, and investment with organisational objectives |
| Digital Policy Development | Élaboration de politiques numériques | Research, consult on, and draft policies governing digital services, data use, and AI within the organisation |
| Enterprise Risk Management | Gestion intégrée des risques | Identify, assess, and manage strategic and operational risks using a framework aligned to Treasury Board policy |
| IT Audit & Assurance | Vérification et assurance des TI | Plan and execute IT control audits; produce findings reports and track management action plan completion |
| Business Continuity & Disaster Recovery | Continuité des activités et reprise après sinistre | Develop, exercise, and maintain BC/DR plans for critical digital services and infrastructure |
| IT Change Governance | Gouvernance des changements TI | Operate a Change Advisory Board process to assess and control risk to live services from planned changes |
| **Privacy & Information Management** | | |
| Privacy & Access to Information (ATIP) | Protection des renseignements personnels et AIPRP | Process ATI and Privacy requests; advise on obligations under the Privacy Act and related legislation |
| Privacy Impact Assessment | Évaluation des facteurs relatifs à la vie privée | Conduct PIAs for new programmes, systems, and data uses; identify and mitigate privacy risks before implementation |
| Information Management | Gestion de l'information | Apply government IM frameworks for creation, capture, classification, retention, and disposal of records |
| Records Management | Gestion des documents | Administer official records per the Library and Archives Canada Act and departmental IM policy |
| Security Classification Management | Gestion de la classification de sécurité | Apply government security classification schemes (Protected A/B/C, Secret, Top Secret) to information assets |
| Open Government & Transparency | Gouvernement ouvert et transparence | Manage proactive disclosure, open data publishing, and departmental transparency reporting obligations |
| **Corporate Governance** | | |
| Corporate Planning & Performance Reporting | Planification et rapports de performance organisationnels | Lead departmental planning cycles (DP, DRR, MAF) and produce evidence-based results and performance reports |
| Treasury Board Submissions & Central Agency Relations | Présentations au CT et relations avec les organismes centraux | Develop TB submissions, Memoranda to Cabinet; manage relationships with PCO, TBS, and Finance Canada |
| Internal Audit | Vérification interne | Plan and execute internal audits per the TB Policy on Internal Audit; report to the Departmental Audit Committee |
| Programme Evaluation | Évaluation des programmes | Design and conduct evaluations per the TB Policy on Results; assess relevance, effectiveness, and efficiency |
| Legal & Policy Interpretation | Interprétation juridique et des politiques | Interpret legislation, regulations, and policy instruments to advise programme and digital service delivery teams |
| Official Languages Management | Gestion des langues officielles | Ensure compliance with the Official Languages Act in service delivery, staffing, and workplace communications |

---

## 17 · Corporate Services

*Finance, HR operations, communications, and administrative support for running the organisation.*  
**Consolidated from**: `Finance` + `HumanResources` + `Communications` + `Administration`.  
**Removed**: `Audit` from Finance (single home: Governance domain); `ATIP` from Administration (single home: Governance domain); `Budgeting` from Administration (duplicate of Government Budgeting in Finance); `HR Processing` from Administration (covered by HR operational skills below).

| name_en | name_fr | description_en |
|---|---|---|
| **Finance** | | |
| Accounting | Comptabilité | Maintain accurate financial records; prepare journal entries, reconciliations, and financial statements |
| Financial Forecasting | Prévision financière | Develop and maintain multi-year financial forecasts; identify variances and advise on corrective action |
| Government Budgeting | Budgétisation gouvernementale | Manage departmental A-base and project budgets in compliance with TB financial management policies |
| Financial Analysis & Reporting | Analyse et rapports financiers | Produce financial analysis and management reports to support executive decision-making and accountability |
| **Human Resources – Operational** | | |
| Staffing | Dotation en personnel | Apply public service staffing rules and appointment processes to fill positions efficiently and equitably |
| Classification | Classification | Evaluate positions against work description standards; assign group and level per TB classification policy |
| Recruiting & Talent Acquisition | Recrutement et acquisition de talents | Source, attract, and assess candidates; manage end-to-end hiring including equity and inclusion considerations |
| Pay & Compensation | Rémunération et avantages sociaux | Administer pay, benefits, and compensation per collective agreements and Treasury Board policy |
| Learning & Development | Apprentissage et perfectionnement | Design, procure, and administer training programmes; manage LMS and mandatory training compliance |
| Equity, Diversity & Inclusion | Équité, diversité et inclusion | Implement EDI action plans; remove systemic hiring barriers; report on workforce representation targets |
| Employee Relations | Relations avec les employés | Administer collective agreements, manage grievances, and advise managers on labour relations obligations |
| Occupational Health & Safety | Santé et sécurité au travail | Administer OHS obligations, workplace accommodation, ergonomic assessments, and return-to-work programmes |
| **Communications** | | |
| Writing & Editing | Rédaction et révision | Produce clear, accurate written products: briefing notes, reports, correspondence, and policy documents |
| Public Speaking & Presentation | Prise de parole et présentation | Deliver confident, clear presentations to internal and external audiences including senior officials and media |
| Media Relations | Relations avec les médias | Manage journalist relationships; prepare lines to take, press releases, and media spokesperson support |
| Strategic Communications | Communications stratégiques | Develop and execute communications plans aligned to programme and organisational objectives |
| **Administration** | | |
| Travel & Logistics Administration | Administration des voyages et de la logistique | Coordinate official travel, accommodation, and logistics in compliance with the National Joint Council Travel Directive |
| Administrative Operations | Opérations administratives | Manage calendars, meeting preparation, correspondence tracking, and day-to-day office operations |

---

## Summary: What Changed from Previous Draft

| Previous domain(s) | Disposition |
|---|---|
| `Leadership` (5 skills) | Merged into `PeopleAndOrganisationalLeadership`; Foresight removed |
| `Management` (4+3 skills) | Merged into `PeopleAndOrganisationalLeadership`; Action Mgmt, Financial Mgmt removed |
| `OrganisationalLeadership` (13 skills) | Merged into `PeopleAndOrganisationalLeadership`; Managing Through Ambiguity, Succession Planning removed |
| `HumanResources` (4+5 skills) | Merged into `CorporateServices` |
| `Finance` (4 skills) | Merged into `CorporateServices`; Audit removed (now in Governance) |
| `Communications` (4 skills) | Merged into `CorporateServices`; Writing renamed Writing & Editing |
| `Administration` (5 skills) | Merged into `CorporateServices`; ATIP → Governance; Budgeting → removed (dup); HR Processing → removed (dup) |
| `InformationTechnology` (7 skills) | Split: software skills → `SoftwareEngineering`; cloud/platform → `CloudPlatformDevOps` |
| `DevOpsAndPlatform` (10 skills) | Merged into `CloudPlatformDevOps`; GitOps kept (distinct from CI/CD); Chaos Engineering removed |
| `AgileDelivery` (12 skills) | Renamed `AgileAndDelivery`; OCM → Leadership domain; Benefits & OKR Tracking → removed (dup with PM) |
| `ProcurementAndContracting` (12 skills) | Merged into `ProcurementAndVendorManagement`; Procurement Compliance & Audit removed |
| `VendorManagement` (10 skills) | Merged into `ProcurementAndVendorManagement`; Escrow, Vendor Consolidation removed; SLA+Perf Reviews merged |
| `Intelligence: Data Analysis` | Removed — Statistical Analysis in DataAnalyticsAndAI is its home |
| `Strategy: Intelligence Analysis` | Removed — All-Source Intelligence Analysis in Intelligence is its home |
| `CyberSecurity: Governance, Risk & Compliance` | Renamed `Information Security Governance` to distinguish from the Governance domain |

---

## Implementation Notes

### New enum variants required (PostgreSQL migration)

```sql
-- New domains
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'SoftwareEngineering';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'CloudPlatformDevOps';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'DataAnalyticsAndAI';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'CyberSecurity';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'ProductManagement';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'AgileAndDelivery';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'UserExperience';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'ProcurementAndVendorManagement';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'PeopleAndOrganisationalLeadership';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'Governance';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'CorporateServices';

-- Existing domains retired by migration of their skills to new domains:
-- Leadership, Management, HumanResources, Finance, Communications,
-- Administration, InformationTechnology
-- These should be marked retired_at in skill records, not deleted,
-- to preserve historical capability data.
```

### Suggested dummy-data domain weights

| Domain | Weight | Rationale |
|---|---|---|
| `Combat` | 15% | Core military population |
| `Engineering` | 8% | Military engineers |
| `Strategy` | 7% | Planners and staff officers |
| `Intelligence` | 5% | Intelligence analysts |
| `Medical` | 5% | Medical personnel |
| `JointOperations` | 3% | Joint staff |
| `SoftwareEngineering` | 8% | Largest civilian tech cohort |
| `CloudPlatformDevOps` | 7% | Infrastructure and platform |
| `DataAnalyticsAndAI` | 6% | Data professionals |
| `CyberSecurity` | 4% | Security specialists |
| `AgileAndDelivery` | 5% | Delivery and programme teams |
| `ProductManagement` | 4% | Product managers |
| `UserExperience` | 3% | UX and design |
| `ProcurementAndVendorManagement` | 3% | Commercial and procurement |
| `PeopleAndOrganisationalLeadership` | 5% | Leaders and managers |
| `Governance` | 3% | Governance and policy |
| `CorporateServices` | 9% | Finance, HR, comms, admin |

### Bilingual descriptions
`name_en` and `name_fr` are provided for all 197 skills. French `description_fr` fields require review by a bilingual editor before production use.
