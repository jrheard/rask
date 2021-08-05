Overall Goal
============
My Taskwarrior RPi setup, with more bells and whistles and less pain (for the user, anyway).
Written in Rust for learning, overengineered also for learning.

Use Cases
=========
- [X] Enter a task
- [X] Show list of active tasks
- [X] Look up a task by ID
- [X] Tasks have numeric IDs
- [X] Mark a task completed
- [X] Tasks can have an optional project
- [X] Tasks can have an optional priority
- [X] Tasks can have an optional due date
- [X] Tasks can be modified a la `task modify`
- [ ] Tasks can be edited a la `task edit` (done in api)
- [ ] Tasks can be uncompleted
- [ ] Tasks can be deleted
- [ ] Multiple machines can view/edit the same task list without needing to carefully merge/sync their state
- [X] Nobody but me can edit/view my tasks
- [ ] Tasks can be marked as "waiting", and are hidden from task list view until they're un-waited
- [ ] Tasks have an "urgency" value that's derived from their various attributes using a simple formula
- [ ] Tasks can be marked as "recurring", eg every two days, two weeks, two months; these tasks recur at midnight on the morning of a particular day, regardless of the time of day when they were created
- [ ] "Waiting" a recurring task is easy/natural (the task is hidden for eg 6 months, and then the recurrence starts as normal afterward)
- [ ] Different machines can specify different display-time profiles with a .raskrc file (primarily affects which columns are shown - what else?)
- [ ] Some sort of `task gc` system automatically "compresses" tasks' separate numeric "display" IDs
- [ ] `task undo`

Stretch Use Cases
=================
- [ ] Some way to view tasks from my phone
- [ ] Some way to enter a task from my phone
- [ ] Some way to mark a task deleted from my phone
- [ ] Stats / burndown charts

Development Setup
=================
- [X] Tests run in CI
- [X] Test coverage is generated in CI
- [X] When developing the CLI and API together, changes to the API are automatically "deployed" to wherever the dev CLI is pointing to
- [ ] Manual deploy to production
- [ ] CD to production

Stretch Overengineering Exercises
=================================
- [ ] Kubernetes? (maybe with Azure; google/aws cost $73/month for a k8s cluster)
- [ ] Terraform?
- [ ] ECS instead?
- [ ] Heroku instead?
- [ ] Logging? (TODO: can/should authorization header values be kept out of logs?)
- [ ] Monitoring?
- [ ] Alerting?

High-Level Design
=================
* Users primarily interact with the system via a CLI similar to `task`
* The CLI makes HTTP requests to a web API
* The system's data is stored in a centralized postgres db
* Recurring tasks are created by a simple daemon process, it also un-waits tasks

-------

Slightly-more-zoomed-in design scratchpad
=========================================

Cargo workspace
* 1 lib crate with shared business logic
* 1 bin crate for web api
* 1 bin crate for recurrence/unwait daemon
    talks to the web api for reads/writes
    TODO read https://www.badykov.com/rust/2020/06/28/you-dont-need-background-job-library/
        waitaminute, why does he even use tokio here? what's the point of bringing async/await into the mix in this situation? no reason imo!
* 1 bin crate for CLI
    talks to the web api for reads/writes

docker containers
    * 1 docker image for http api crate
    * 1 docker image for recurrence/unwait daemon crate
    * 1 docker image for db
        * 1 volume for persistence

authentication for CLI done via API token

configuration for CLI will be something to figure out - probably a .raskrc?
it'll have to know which IP to point at for the web api backend, and it'll have to know about api token

no clue how i'll do authentication for phone
    if i allow mobile devices to access the system via an html webapp, how do i store the api token on the phone?
    if i only allow tasks to be entered via phone, i could just use email and have only my own email addresses whitelisted
        wouldn't be able to view / complete tasks via phone, but that'd be totally fine imo