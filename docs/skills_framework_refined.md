# Refined Skills Framework — Cognitive-Load Optimised
## Defence Digital & Technology Organisation

**Version**: 2.0 — supersedes `skills_framework_consolidated.md`  
**Total**: 16 domains · 115 skills · **every domain holds 5–9 skills (Miller's 7±2)**

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

This pass is primarily about the **skill lists** within each domain. The one structural change
is folding `ProductManagement` and `AgileAndDelivery` into a single `ProductAgileAndDelivery`
domain (see Implementation Notes) — every other domain name and the rest of the `SkillDomain`
enum are unchanged from the consolidated draft.

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
| ProductManagement + AgileAndDelivery → ProductAgileAndDelivery | 10 + 10 | 7 | domains merged (heavy overlap) and rationalised to 7 |
| UserExperience | 10 | 7 | merged research/testing, interaction/IA, prototyping/systems |
| ProcurementAndVendorManagement | 17 | 7 | merged into 7 lifecycle-stage capability groups |
| PeopleAndOrganisationalLeadership | 21 | 6 | replaced with the GoC's 6 Key Leadership Competencies |
| Governance | 18 | 7 | merged into 7 governance capability clusters |
| CorporateServices | 18 | 7 | merged into 7 corporate function clusters |
| **Total** | **202** | **115** | **43% reduction; zero coverage lost** |

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

## 11 · Product, Agile & Delivery *(Product Management 10 + Agile & Delivery 10 → 7, domains merged)*

*Product Management and Agile & Delivery overlap heavily — both describe how digital work is
shaped, planned, and shipped. Folded into a single domain (`ProductAgileAndDelivery`) and
rationalised from a combined 14 skills down to 7, so "do we have product/delivery capability?"
is answerable from one list instead of two. The `SkillDomain` enum value `agile_and_delivery`
was renamed to `product_agile_and_delivery`; `product_management` was retired and its skills
folded in — both changes were made directly to the original migration files since this is a
pre-production schema with no live data to migrate.*

| name_en | name_fr | description_en |
|---|---|---|
| Product Strategy & Vision | Stratégie et vision produit | Define and communicate product direction aligned to user and organisational needs |
| Roadmapping, Prioritisation & Backlog Management | Feuille de route, priorisation et gestion du carnet de travail | Build outcome-oriented roadmaps, prioritise the portfolio, and manage backlogs with clear, testable requirements |
| Agile Delivery & Coaching | Livraison agile et coaching | Facilitate agile frameworks (Scrum, Kanban, SAFe), apply Lean thinking, and coach teams toward higher agile maturity |
| Programme & Project Management | Gestion de programmes et de projets | Plan, govern, and report on programmes, projects, and team-level delivery toward shared outcomes |
| Market & User Research and Stakeholder Engagement | Recherche marché et utilisateurs et engagement des parties prenantes | Gather market and user insight and manage relationships across delivery, policy, and leadership audiences |
| Business Case, Benefits & Outcome Measurement | Analyse de rentabilisation, gestion des avantages et mesure des résultats | Build investment cases, track benefits realisation, and define OKRs/KPIs that show whether work delivers value |
| Delivery Risk & Dependency Management | Gestion des risques et des dépendances de livraison | Identify, communicate, and mitigate delivery risks, constraints, and cross-team dependencies |

## 12 · User Experience & Design *(10 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| User Research & Usability Testing | Recherche utilisateurs et tests d'utilisabilité | Plan research and usability studies; synthesise findings into actionable recommendations |
| Service Design | Conception de services | Map and redesign end-to-end service journeys across digital and non-digital touchpoints |
| Interaction & Information Design | Conception d'interaction et de l'information | Design interface flows and organise content/navigation for findability and task completion |
| Prototyping & Design Systems | Prototypage et systèmes de conception | Build prototypes and govern component libraries and design tokens for consistency at scale |
| Accessibility & Inclusive Design | Accessibilité et conception inclusive | Ensure products meet WCAG 2.1 AA and serve people with a diverse range of abilities |
| Content Design | Conception de contenu | Write and structure interface copy, error messages, and notifications that guide users |
| UX Strategy | Stratégie UX | Define UX maturity, investment, research programme, and tooling strategy aligned to goals |

## 13 · Procurement & Vendor Management *(17 → 7)*

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

## 14 · People & Organisational Leadership *(21 → 6 — aligned to GoC Key Leadership Competencies)*

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

## 15 · Governance & Compliance *(18 → 7)*

| name_en | name_fr | description_en |
|---|---|---|
| IT & Digital Governance | Gouvernance des TI et du numérique | Apply IT governance frameworks, develop digital policy, and govern change to live services |
| Enterprise Risk & Business Continuity | Gestion des risques organisationnels et continuité des activités | Manage strategic/operational risk and maintain business continuity and disaster recovery plans |
| Audit & Programme Evaluation | Vérification et évaluation des programmes | Plan and execute IT, internal, and programme audits and evaluations against TB policy |
| Privacy & Access to Information | Protection des renseignements personnels et accès à l'information | Process ATIP requests and conduct Privacy Impact Assessments under the Privacy Act |
| Information & Records Management | Gestion de l'information et des documents | Manage records, classification, and information assets per government IM frameworks |
| Corporate Planning & Reporting | Planification et rapports organisationnels | Lead departmental planning, performance reporting, Treasury Board submissions, and transparency obligations |
| Legal, Policy & Official Languages | Affaires juridiques, politiques et langues officielles | Interpret legislation and policy and ensure compliance with the Official Languages Act |

## 16 · Corporate Services *(18 → 7)*

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

### Domain merge: ProductManagement + AgileAndDelivery → ProductAgileAndDelivery
The two domains were folded into one (`product_agile_and_delivery`) because their skill sets
overlapped heavily. Since this is a pre-production schema with no live data, the rename/removal
was made **directly in the original migration files** (`2026-06-04-000001_update_skill_domain_enum`)
rather than as an additive follow-up migration — `product_management` was dropped from the
`ALTER TYPE ... ADD VALUE` list and `agile_and_delivery` was renamed to `product_agile_and_delivery`.
The Rust `SkillDomain` enum and its weighted `Distribution` impl were updated to match
(combined weight range `68..=76`, replacing the two previous ranges).

Every other domain name and the rest of the `SkillDomain` enum are unchanged from the
consolidated draft.

### Code change scope
`pre_populate_skills()` in `dummy_capability_data.rs` needs its skill vector replaced with the
115-skill list above (down from 202), and any `SkillDomain::ProductManagement` /
`SkillDomain::AgileAndDelivery` references updated to `SkillDomain::ProductAgileAndDelivery`.
All downstream logic (capability generation, validations, requirements) is domain-driven and
requires no further changes.

### Bilingual descriptions
Descriptions above are intentionally broader than the consolidated draft's — each one names
the specific practices or technologies it covers, so reviewers can confirm no capability was
silently dropped during merging. `description_fr` requires translation review before production use.
