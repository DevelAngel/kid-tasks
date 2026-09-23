use anyhow::Result;
use cucumber::{then, when};
use thirtyfour::prelude::*;

use crate::world::AppWorld;

#[when("I enable edit mode")]
async fn enable_edit_mode(world: &mut AppWorld) -> Result<()> {
    let button = world.http.find(By::Testid("edit-mode-toggle")).await?;
    button.click().await?;
    Ok(())
}


#[when(expr = "I type {string} and submit")]
async fn type_and_submit(world: &mut AppWorld, text: String) -> Result<()> {
    let input = world
        .http
        .query(By::Css("input[placeholder='New task…']"))
        .first()
        .await?;
    input.send_keys(&text).await?;
    input.send_keys(Key::Enter).await?;
    // Brief wait for the server round-trip and re-render.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok(())
}

#[then("the new task is expanded")]
async fn task_is_expanded(world: &mut AppWorld) -> Result<()> {
    world
        .http
        .query(By::Testid("task-details"))
        .first()
        .await?;
    Ok(())
}

#[then("the Add Task input is visible in the viewport")]
async fn add_task_input_in_viewport(world: &mut AppWorld) -> Result<()> {
    let input = world
        .http
        .query(By::Css("input[placeholder='New task…']"))
        .first()
        .await?;
    let in_viewport: bool = world
        .http
        .execute(
            r#"
            const rect = arguments[0].getBoundingClientRect();
            return rect.top >= 0
                && rect.left >= 0
                && rect.bottom <= window.innerHeight
                && rect.right <= window.innerWidth;
            "#,
            vec![input.to_json()?],
        )
        .await?
        .convert()?;
    assert!(in_viewport, "Add Task input is not visible in the viewport");
    Ok(())
}

#[then("the Add Task input has focus")]
async fn add_task_input_has_focus(world: &mut AppWorld) -> Result<()> {
    let input = world
        .http
        .query(By::Css("input[placeholder='New task…']"))
        .first()
        .await?;
    let focused: bool = world
        .http
        .execute(
            "return document.activeElement === arguments[0];",
            vec![input.to_json()?],
        )
        .await?
        .convert()?;
    assert!(focused, "Add Task input does not have focus");
    Ok(())
}


#[then("no Add Task error is shown")]
async fn no_add_task_error(world: &mut AppWorld) -> Result<()> {
    let result = world
        .http
        .query(By::Testid("add-task-error"))
        .nowait()
        .first_opt()
        .await?;
    if let Some(el) = result {
        let text = el.text().await?;
        panic!("expected no error, but found: {text}");
    }
    Ok(())
}

#[then("I see an Add Task error")]
async fn see_add_task_error(world: &mut AppWorld) -> Result<()> {
    let el = world
        .http
        .query(By::Testid("add-task-error"))
        .first()
        .await?;
    let text = el.text().await?;
    assert!(
        text.starts_with("Task could not be created"),
        "expected error starting with 'Task could not be created', got: {text}",
    );
    Ok(())
}
