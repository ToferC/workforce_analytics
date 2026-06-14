use async_graphql::Error;
use rand::{seq::SliceRandom, Rng};
use uuid::Uuid;

use crate::models::{CapabilityLevel, NewProduct, NewWork, Product, Skill, Task, Work, WorkStatus};

/// Generate dummy products for an organization, attach existing tasks to
/// them and plan vacant work elements with capability requirements so the
/// capability matching flow can be demonstrated end to end.
pub fn generate_dummy_products(organization_id: &Uuid) -> Result<(), Error> {

    let mut rng = rand::thread_rng();

    let product_names: Vec<&str> = "
        Readiness Dashboard; Talent Rostering Platform; Logistics Tracking System;
        Field Operations Portal; Integrated Performance Measurement System
    ".split("; ").collect();

    let work_verbs: Vec<&str> = "
        design; build; test; document; deliver; review
    ".split("; ").collect();

    let mut tasks = Task::get_all()?;
    tasks.shuffle(&mut rng);

    println!("Creating Products with vacant work");

    for name in product_names {

        // Attach a handful of existing tasks to the product
        let attach_count = rng.gen_range(2..=4).min(tasks.len());

        if attach_count == 0 {
            break;
        }

        let product_tasks: Vec<Task> = tasks.drain(0..attach_count).collect();

        // The creator of the product's first task owns the product
        let np = NewProduct::new(
            *organization_id,
            product_tasks[0].created_by_role_id,
            name.trim().to_string(),
            format!("{}_FR", name.trim()),
            "Description_EN".to_string(),
            "Description_FR".to_string(),
            product_tasks[0].domain,
            Some("https://www.forces.ca/some_url".to_string()),
            WorkStatus::InProgress,
        );

        let product = Product::create(&np)?;

        let mut vacant_work = Vec::new();

        for mut task in product_tasks {

            task.product_id = Some(product.id);
            let task = task.update()?;

            let domain_skills = Skill::get_by_domain(task.domain)?;

            // Plan unassigned work under each task, half targeting a
            // specific skill, awaiting a capability match
            for _ in 0..rng.gen_range(1..=2) {

                let skill_id = domain_skills.choose(&mut rng)
                    .filter(|_| rng.gen_bool(0.5))
                    .map(|s| s.id);

                let nw = NewWork::new(
                    task.id,
                    None,
                    format!("{} {}",
                        work_verbs.choose(&mut rng).unwrap().trim(),
                        task.title.trim()),
                    Some("https://www.forces.ca/some_url".to_string()),
                    task.domain,
                    skill_id,
                    rand::random::<CapabilityLevel>(),
                    rng.gen_range(1..=3),
                    WorkStatus::Planning,
                );

                vacant_work.push(nw);
            }
        }

        let _r = Work::batch_create(&vacant_work)?;
    }

    Ok(())
}
