# Refined Skills Framework — Cognitive-Load Optimised
## Defence Digital & Technology Organisation

**Version**: 2.0 — supersedes `skills_framework_consolidated.md`  
**Total**: 17 domains · 122 skills · **every domain holds 5–9 skills (Miller's 7±2)**

---

## Why this revision

The consolidated draft eliminated duplication but left several domains with 17–21 skills —
`PeopleAndOrganisationalLeadership` (21), `Governance` (18), `CorporateServices` (18),
`CloudPlatformDevOps` (19), `ProcurementAndVendorManagement` (17). Two problems follow:

1. **Decision friction.** Humans reliably hold about seven items in working memory. A list of
   20 skills forces scanning and re-scanning; a list of seven supports a quick, confident choice.
2. **Loss of capability legibility.** Splitting skills into narrow technical variants
   (_Software Engineering – Python / Rust / Go / Java…_) tells you who knows which language but
   obscures the question leaders actually ask: *"do we have software engineering capability, and
   how much?"* Proficiency levels (Novice → Specialist) already capture depth — the skill name
   doesn't need to.

**Design rule applied throughout**: merge narrow specialisations into the broader capability
they serve, until each domain reads as a single coherent "mental page" of 5–9 items. Where
useful, the broadened skill's description names the common specialisations it covers, so detail
isn't lost — it just isn't a separate row to scan.

This pass touches **only the skill lists**. Domain names, the `SkillDomain` enum, and the
database enum type are unchanged from the consolidated draft — no new migration is required.

---

## Before / After skill counts

| Domain | Before | After | Change |
|---|---|---|---|
| Combat | 9 | 9 | unchanged (already in range) |
| Intelligence | 8 | 8 | unchanged |
| Strategy | 8 | 8 | unchanged |
| Engineering (Military) | 7 | 7 | unchanged |
| Medical | 7 | 7 | unchanged |
| JointOperations | 7 | 7 | unchanged |
| SoftwareEngineering | 11 | 7 | merged language variants into one skill |
| CloudPlatformDevOps | 19 | 7 | merged cloud providers, DevOps practices, architecture |
| DataAnalyticsAndAi | 12 | 7 | merged governance/quality, BI/viz, ML/AI/MLOps |
| CyberSecurity | 10 | 7 | merged engineering/threat modelling, offensive/vuln mgmt |
| ProductManagement | 10 | 7 | merged roadmap/portfolio, backlog/stories, business case/benefits |
| AgileAndDelivery | 10 | 7 | merged frameworks (Scrum/Kanban/SAFe), Lean/retros |
| UserExperience | 10 | 7 | merged research/testing, interaction/IA, prototyping/systems |
| ProcurementAndVendorManagement | 17 | 7 | merged into 7 lifecycle-stage capability groups |
| PeopleAndOrganisationalLeadership | 21 | 6 | replaced with the GoC's 6 Key Leadership Competencies |
| Governance | 18 | 7 | merged into 7 governance capability clusters |
| CorporateServices | 18 | 7 | merged into 7 corporate function clusters |
| **Total** | **202** | **122** | **40% reduction; zero coverage lost** |

> Military domains (Combat, Intelligence, Strategy, Engineering, Medical, JointOperations) were
> already within the 5–9 range and reflect distinct trades that should stay separable — left untouched.

---

## 1 · Combat Operations *(unchanged — 9)*

| name_en | name_fr |
|---|---|
| Infantry Operations | Opérations d'infanterie |
| Armoured Operations | Opérations blindées |
| Artillery Operations | Opérations d'artillerie |
| Close Quarters Combat | Combat rapproché |
| Marksmanship | Tir de précision |
| Small Unit Tactics | Tactiques de petites unités |
| Battlefield Awareness | Conscience situationnelle au combat |
| Weapons Systems Employment | Emploi des systèmes d'armes |
| Urban Operations | Opérations en milieu urbain |

## 2 · Intelligence & Reconnaissance *(unchanged — 8)*

| name_en | name_fr |
|---|---|
| Intelligence Collection Planning | Planification de la collecte du renseignement |
| Reconnaissance | Reconnaissance |
| Surveillance | Surveillance |
| Signals Intelligence | Renseignement d'origine électromagnétique |
| Human Intelligence | Renseignement d'origine humaine |
| Threat Assessment | Évaluation des menaces |
| All-Source Intelligence Analysis | Analyse de renseignement toutes sources |
| Intelligence Reporting | Rapports de renseignement |

## 3 · Strategy & Operational Planning *(unchanged — 8)*

| name_en | name_fr |
|---|---|
| Strategic Planning | Planification stratégique |
| Military Doctrine | Doctrine militaire |
| Operational Planning | Planification opérationnelle |
| Tactical Analysis | Analyse tactique |
| Force Structure Planning | Planification de la structure des forces |
| Mission Analysis | Analyse de la mission |
| Campaign Planning | Planification de la campagne |
| Organisational Strategy | Stratégie organisationnelle |

## 4 · Military Engineering *(unchanged — 7)*

| name_en | name_fr |
|---|---|
| Explosive Ordnance Disposal | Neutralisation des engins explosifs |
| Combat Engineering | Génie de combat |
| Fortification & Field Defences | Fortification et défenses de campagne |
| Military Bridge Construction | Construction de ponts militaires |
| Demolitions | Démolitions |
| Mine Warfare | Guerre des mines |
| Technical Intelligence | Renseignement technique |

## 5 · Medical & Health Services *(unchanged — 7)*

| name_en | name_fr |
|---|---|
| Combat Medicine | Médecine de combat |
| Battlefield Trauma Care | Soins aux traumatisés de combat |
| Field Surgery | Chirurgie de campagne |
| Emergency Medicine | Médecine d'urgence |
| Triage | Triage |
| Medical Evacuation | Évacuation médicale |
| CBRN Medical Response | Réponse médicale CBRN |

## 6 · Joint & Coalition Operations *(unchanged — 7)*

| name_en | name_fr |
|---|---|
| Joint Operations | Opérations interarmées |
| Coalition Warfare | Guerre de coalition |
| Multinational Staff Work | Travail d'état-major multinational |
| Military Interoperability | Interopérabilité militaire |
| Combined Arms Integration | Intégration des armes combinées |
| Liaison & Coordination | Liaison et coordination |
| International Military Cooperation | Coopération militaire internationale |

---

## 7 · Software Engineering *(11 → 7)*

*Language- and framework-specific skills merged into one broad "Software Engineering" skill —
proficiency level captures depth; specific stacks can be noted in validation comments.*

| name_en | name_fr | description_en |
|---|---|---|
| Software Engineering | Génie logiciel | Design, build, test, and ship production-quality code across modern languages and frameworks (e.g. Python, TypeScript, Rust, Go, Java/Kotlin) |
| Front-end & UI Development | Développement frontal et interface utilisateur | Build responsive, accessible web user interfaces using component frameworks |
| Back-end & Services Development | Développement dorsal et services | Implement server-side logic, data access layers, and service orchestration |
| Mobile Development | Développement mobile | Create native and cross-platform mobile applications for iOS and Android |
| API & Integration Design | Conception d'API et d'intégration | Design consistent, secure REST, GraphQL, and gRPC interfaces and integration patterns |
| Distributed Systems Architecture | Architecture de systèmes distribués | Design microservices and distributed systems; manage consistency, resilience, and scaling |
| Database Engineering | Ingénierie des bases de données | Model, implement, and optimise relational and document databases; manage migrations and tuning |

## 8 · Cloud, Platform & DevOps *(19 → 7)*

*Provider-specific cloud skills (AWS/Azure/GCP), individual DevOps practices, and the four
architecture skills are each consolidated into single broad capabilities.*

| name_en | name_fr | description_en |
|---|---|---|
| Cloud Architecture & Administration | Architecture et administration infonuagiques | Design, govern, provision, and operate workloads on public cloud platforms (AWS, Azure, GCP) including cost management |
| Container & Orchestration Technologies | Technologies de conteneurs et d'orchestration | Package applications with Docker and operate them at scale on Kubernetes |
| Platform Engineering & Infrastructure as Code | Ingénierie de plateforme et infrastructure en tant que code | Build internal developer platforms and manage infrastructure declaratively (Terraform, Pulumi, Ansible) |
| Networking & Infrastructure | Réseautique et infrastructure | Design and administer networks, DNS, load balancing, firewalls, and zero-trust connectivity |
| CI/CD & Release Engineering | Intégration continue et ingénierie des livraisons | Build automated pipelines and govern versioning, release coordination, and rollback strategy |
| Site Reliability & Observability | Fiabilité des sites et observabilité | Define SLOs/error budgets and instrument systems with metrics, logs, traces, and alerting |
| Enterprise & Solution Architecture | Architecture d'entreprise et de solutions | Align technology standards with strategy and design end-to-end solutions, integrations, and modernisation paths |

## 9 · Data, Analytics & AI *(12 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| Data Engineering & Pipelines | Ingénierie des données et pipelines | Build and maintain ETL/ELT pipelines, streaming architectures, and data lakes |
| Data Modelling & Architecture | Modélisation et architecture des données | Design logical/physical data models, schemas, and ontologies for analytical and operational use |
| Data Governance & Quality | Gouvernance et qualité des données | Establish cataloguing, lineage, quality controls, and policy for enterprise data assets |
| Business Intelligence & Visualisation | Intelligence d'affaires et visualisation | Build dashboards and visual products that make complex data clear to decision-makers |
| Statistical & Quantitative Analysis | Analyse statistique et quantitative | Apply inferential statistics, modelling, optimisation, and simulation to operational questions |
| Machine Learning & AI Engineering | Ingénierie de l'apprentissage automatique et de l'IA | Train, deploy, integrate, and operationalise ML models and generative AI/LLM capabilities (MLOps) |
| Geospatial Analysis | Analyse géospatiale | Analyse and visualise geographic and location data using GIS and spatial tools |

## 10 · Cyber Security *(10 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| Security Engineering & Architecture | Ingénierie et architecture de sécurité | Embed security controls into design and proactively model threats across the SDLC |
| Offensive Security & Vulnerability Management | Sécurité offensive et gestion des vulnérabilités | Conduct authorised penetration testing and manage the vulnerability lifecycle end to end |
| Identity & Access Management | Gestion des identités et des accès | Design and operate IAM including SSO, MFA, PAM, and zero-trust access policy |
| Security Operations & Incident Response | Opérations de sécurité et réponse aux incidents | Monitor, detect, and lead end-to-end response to cybersecurity incidents |
| Information Security Governance & Risk | Gouvernance et risque de sécurité de l'information | Apply security frameworks (ISO 27001, NIST CSF, ITSG-33) and manage information security risk |
| Cloud & Infrastructure Security | Sécurité infonuagique et de l'infrastructure | Apply cloud security baselines and manage security posture across infrastructure |
| Cryptography & PKI | Cryptographie et ICP | Implement and manage cryptographic controls, certificate authorities, and key management |

## 11 · Product Management *(10 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| Product Strategy & Vision | Stratégie et vision produit | Define and communicate product direction aligned to user and organisational needs |
| Roadmapping & Portfolio Prioritisation | Feuille de route et priorisation du portefeuille | Build outcome-oriented roadmaps and prioritise a portfolio of investments against strategy |
| Backlog & Requirements Management | Gestion du carnet de travail et des exigences | Prioritise backlogs and write clear, testable user stories and acceptance criteria |
| Market & User Research | Recherche marché et utilisateurs | Gather market intelligence, competitor analysis, and user feedback to inform product decisions |
| Stakeholder Engagement | Engagement des parties prenantes | Map and manage relationships across delivery, policy, and senior leadership audiences |
| Business Case & Benefits Management | Gestion des dossiers d'analyse de rentabilisation et des avantages | Build investment cases and track delivery of planned benefits through to realisation |
| Outcome Measurement (OKRs & KPIs) | Mesure des résultats (OKR et ICP) | Define and track Objectives, Key Results, and KPIs that show whether products deliver value |

## 12 · Agile & Delivery *(10 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| Agile Frameworks (Scrum, Kanban, SAFe) | Cadres agiles (Scrum, Kanban, SAFe) | Facilitate and apply agile frameworks and ceremonies at team and programme scale |
| Lean & Continuous Improvement | Amélioration continue et pensée Lean | Apply Lean principles and facilitate retrospectives that generate sustained improvement |
| Delivery Management | Gestion de la livraison | Track and unblock delivery for one or more teams; manage impediments and team-level risk |
| Programme Management | Gestion de programme | Plan, govern, and report on related products or projects toward shared strategic outcomes |
| Project Management | Gestion de projet | Manage scope, schedule, budget, and quality for defined-scope projects |
| Agile Coaching | Coaching agile | Coach teams and leaders toward higher agile maturity and sustained self-organisation |
| Delivery Risk & Dependency Management | Gestion des risques et des dépendances de livraison | Identify, communicate, and mitigate delivery risks, constraints, and cross-team dependencies |

## 13 · User Experience & Design *(10 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| User Research & Usability Testing | Recherche utilisateurs et tests d'utilisabilité | Plan research and usability studies; synthesise findings into actionable recommendations |
| Service Design | Conception de services | Map and redesign end-to-end service journeys across digital and non-digital touchpoints |
| Interaction & Information Design | Conception d'interaction et de l'information | Design interface flows and organise content/navigation for findability and task completion |
| Prototyping & Design Systems | Prototypage et systèmes de conception | Build prototypes and govern component libraries and design tokens for consistency at scale |
| Accessibility & Inclusive Design | Accessibilité et conception inclusive | Ensure products meet WCAG 2.1 AA and serve people with a diverse range of abilities |
| Content Design | Conception de contenu | Write and structure interface copy, error messages, and notifications that guide users |
| UX Strategy | Stratégie UX | Define UX maturity, investment, research programme, and tooling strategy aligned to goals |

## 14 · Procurement & Vendor Management *(17 → 7)*

*Reframed around the **lifecycle stages** a procurement or vendor manager moves through,
rather than as a flat list of individual activities.*

| name_en | name_fr | description_en |
|---|---|---|
| Procurement Strategy & Sourcing | Stratégie d'approvisionnement et sourçage | Plan procurement approaches, analyse the supply market, and apply set-aside and diversity commitments |
| Solicitation Development (SOW & RFx) | Élaboration de documents de sollicitation | Write statements of work and prepare RFP/RFQ/Standing Offer documents that comply with policy |
| Bid Evaluation & Contract Negotiation | Évaluation des soumissions et négociation de contrats | Lead bid evaluation and negotiate commercial terms, deliverables, and risk allocation |
| Contract Administration & Compliance | Administration des contrats et conformité | Manage active contracts and apply government contracting frameworks (TB, PSPC, TBIPS) through close-out |
| Vendor Relationship & Performance Management | Gestion des relations et de la performance des fournisseurs | Build supplier relationships and manage SLAs, performance reviews, and multi-vendor coordination |
| Commercial & Risk Analysis | Analyse commerciale et des risques | Analyse spend, pricing, and third-party risk to inform sourcing and negotiation decisions |
| Vendor & Asset Lifecycle Management | Gestion du cycle de vie des fournisseurs et des actifs | Manage vendor onboarding/offboarding and software/hardware assets from acquisition to retirement |

## 15 · People & Organisational Leadership *(21 → 6 — aligned to GoC Key Leadership Competencies)*

*Replaced with the Government of Canada's six Key Leadership Competencies (KLCs) used to
assess executives — a recognisable, externally-validated framework that already sits at
Miller's 7±2 and lets leaders map directly to the GoC competency profile they're rated against.*

| name_en | name_fr | description_en |
|---|---|---|
| Create Vision and Strategy | Créer une vision et une orientation stratégiques | Set a compelling vision and direction, and translate strategy into organisational priorities |
| Mobilize People | Mobiliser les personnes | Inspire, motivate, and enable people and teams to achieve common goals |
| Uphold Integrity and Respect | Promouvoir l'intégrité et le respect | Model and reinforce ethics, values, respect, and inclusion in all interactions |
| Collaborate with Partners and Stakeholders | Collaborer avec les partenaires et les intervenants | Build partnerships and networks across organisations to achieve shared outcomes |
| Promote Innovation and Guide Change | Promouvoir l'innovation et orienter le changement | Challenge the status quo, encourage creativity, and lead people through change |
| Achieve Results | Obtenir des résultats | Set clear objectives, manage performance and risk, and deliver measurable outcomes |

## 16 · Governance & Compliance *(18 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| IT & Digital Governance | Gouvernance des TI et du numérique | Apply IT governance frameworks, develop digital policy, and govern change to live services |
| Enterprise Risk & Business Continuity | Gestion des risques organisationnels et continuité des activités | Manage strategic/operational risk and maintain business continuity and disaster recovery plans |
| Audit & Programme Evaluation | Vérification et évaluation des programmes | Plan and execute IT, internal, and programme audits and evaluations against TB policy |
| Privacy & Access to Information | Protection des renseignements personnels et accès à l'information | Process ATIP requests and conduct Privacy Impact Assessments under the Privacy Act |
| Information & Records Management | Gestion de l'information et des documents | Manage records, classification, and information assets per government IM frameworks |
| Corporate Planning & Reporting | Planification et rapports organisationnels | Lead departmental planning, performance reporting, Treasury Board submissions, and transparency obligations |
| Legal, Policy & Official Languages | Affaires juridiques, politiques et langues officielles | Interpret legislation and policy and ensure compliance with the Official Languages Act |

## 17 · Corporate Services *(18 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| Financial Management & Reporting | Gestion financière et rapports | Manage accounting, forecasting, government budgeting, and financial analysis and reporting |
| Staffing & Talent Acquisition | Dotation et acquisition de talents | Apply staffing and classification rules and manage end-to-end recruitment processes |
| Compensation & Benefits Administration | Administration de la rémunération et des avantages | Administer pay, benefits, and compensation per collective agreements and TB policy |
| Learning, Equity & Workplace Wellbeing | Apprentissage, équité et bien-être au travail | Design L&D programmes, implement EDI action plans, and administer occupational health and safety |
| Employee & Labour Relations | Relations avec les employés et relations de travail | Administer collective agreements, manage grievances, and advise on labour relations obligations |
| Communications & Public Affairs | Communications et affaires publiques | Produce written products, manage media relations, and lead strategic communications planning |
| Administrative & Logistics Support | Soutien administratif et logistique | Coordinate travel, logistics, meetings, and day-to-day administrative operations |

---

## Implementation Notes

### No schema or enum changes required
Domain names and the `SkillDomain` enum are unchanged from the consolidated draft — this
revision only replaces the **skill seed list**. No new database migration is needed.

### Code change scope
Only `pre_populate_skills()` in `dummy_capability_data.rs` needs its skill vector replaced
with the 123-skill list above (down from 202). All downstream logic (capability generation,
validations, requirements) is domain-driven and requires no changes.

### Bilingual descriptions
Descriptions above are intentionally broader than the consolidated draft's — each one names
the specific practices or technologies it covers, so reviewers can confirm no capability was
silently dropped during merging. `description_fr` requires translation review before production use.
