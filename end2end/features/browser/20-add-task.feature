Feature: Add Task in edit mode

  The new task input is immediately available in edit mode,
  so the user can create a task without an extra click.

  # ── View switch ────────────────────────────────────────────

  Scenario: Add Task from Upcoming switches to All Open
    Given no tasks at all
    When I open the app in Upcoming view
    And I enable edit mode
    Then I see the page title is All Open

  Scenario: Add Task from Quick Wins switches to All Open
    Given no tasks at all
    When I open the app in Quick Wins view
    And I enable edit mode
    Then I see the page title is All Open

  Scenario: Add Task from All Open stays in All Open
    Given no tasks at all
    When I open the app in All Open view
    And I enable edit mode
    Then I see the page title is All Open

  Scenario: Add Task from What I Finished switches to All Open
    Given no tasks at all
    When I open the app in What I Finished view
    And I enable edit mode
    Then I see the page title is All Open

  Scenario: Add Task from Recent Changes switches to All Open
    Given no tasks at all
    When I open the app in Recent Changes view
    And I enable edit mode
    Then I see the page title is All Open

  # ── Task visible after creation ───────────────────────────

  Scenario: Task created from Upcoming is visible and expanded in All Open
    Given I am logged in as "e2e-test"
    And no tasks at all
    When I open the app in Upcoming view
    And I enable edit mode
    And I type "Buy milk" and submit
    Then no Add Task error is shown
    And I see the page title is All Open
    And I see tasks in the list
    And the new task is expanded

  Scenario: Add Task input is visible and focused with many tasks
    Given a viewport of 360 by 480
    And the following tasks
      | summary  | category | status | days ago |
      | Task 01  | Inbox    | open   | 1        |
      | Task 02  | Inbox    | open   | 1        |
      | Task 03  | Inbox    | open   | 1        |
      | Task 04  | Inbox    | open   | 1        |
      | Task 05  | Inbox    | open   | 1        |
    When I open the app in Upcoming view
    And I enable edit mode
    Then the Add Task input is visible in the viewport
    And the Add Task input has focus

  # ── Error handling ────────────────────────────────────────

  Scenario: Add Task without login shows error
    Given no user is logged in
    And no tasks at all
    When I open the app in All Open view
    And I enable edit mode
    And I type "Buy milk" and submit
    Then I see an Add Task error
