Overall Goal
============
My Taskwarrior RPi setup, with more bells and whistles and less pain (for the user, anyway).
Written in Rust for learning, likely very overengineered also for learning.

Use Cases
=========
- [ ] Enter a task (done in api)
- [ ] Show list of active tasks (done in api)
- [ ] Tasks have numeric IDs (done in api)
- [ ] Mark a task completed (done in api)
- [ ] Tasks can have an optional project (done in api)
- [ ] Tasks can have an optional priority (done in api)
- [ ] Tasks can have an optional due date (done in api)
- [ ] Tasks can be edited a la `task edit`
- [ ] Tasks can be marked as "waiting", and are hidden from task list view until they're un-waited
- [ ] Multiple machines can view/edit the same task list without needing to carefully merge/sync their state
- [ ] Nobody but me can edit/view my tasks
- [ ] Tasks have an "urgency" value that's derived from their various attributes using a simple formula
- [ ] Tasks can be marked as "recurring", eg every two days, two weeks, two months; these tasks recur at midnight on the morning of a particular day, regardless of the time of day when they were created
- [ ] "Waiting" a recurring task is easy/natural (the task is hidden for eg 6 months, and then the recurrence starts as normal afterward)
- [ ] Different machines can specify different display-time profiles with a .raskrc file (primarily affects which columns are shown - what else?)
- [ ] Some sort of `task gc` system automatically "compresses" tasks' separate numeric "display" IDs

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
- [ ] When developing the CLI and API together, changes to the API are automatically "deployed" to wherever the dev CLI is pointing to
- [ ] Manual deploy to production on my linode (blocked on setting up auth first)
- [ ] CD to production

Stretch Overengineering Exercises
=================================
- [ ] Kubernetes (maybe with Azure; google/aws cost $73/month for a k8s cluster)
- [ ] Terraform
- [ ] Monitoring?
- [ ] Alerting?

High-Level Design
=================
* Users primarily interact with the system via a CLI similar to `task`
* The CLI makes HTTP requests to a web API
* The system's data is stored in a centralized postgres db
* Recurring tasks are created by a simple daemon process

-------

Slightly-more-zoomed-in design scratchpad
=========================================

Cargo workspace
* 1 lib crate with shared business logic
    TODO: does this business logic lib crate know how to interact with the DB?
* 1 bin crate for web api
    TODO: is the web api the only part of the system that talks directly to the db?
* 1 bin crate for recurrence daemon
    TODO how does the daemon talk to the db? through the api or directly?
        through the api imo
    TODO read https://www.badykov.com/rust/2020/06/28/you-dont-need-background-job-library/
        waitaminute, why does he even use tokio here? what's the point of bringing async/await into the mix in this situation? no reason imo!
* 1 bin crate for CLI
    talks to the web api for reads/writes

docker-compose setup
    * 1 docker image for http api crate
    * 1 docker image for recurrence daemon crate
    * 1 docker image for db
        * 1 volume for persistence

authentication for CLI done via SSH key - i'll generate a keypair specifically for this app and distribute the private key to my machines
requests to API need to be signed by private SSH key

configuration for CLI will be something to figure out - probably a .raskrc?
it'll have to know which IP to point at for the web api backend

no clue how i'll do authentication for phone.
    if i allow mobile devices to access the system via an html webapp, i could do oauth with a single allowed user?
    if i only allow tasks to be entered via phone, i could just use email and have only my own email addresses whitelisted
        wouldn't be able to view / complete tasks via phone, but that'd be totally fine imo