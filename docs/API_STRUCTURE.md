# API Structure

## models

`models` contains Fosslate's own concepts.

This is where things like projects, namespaces, strings, translations, languages, votes, and approvals live.

Models should mostly describe what something is. They should not know how to talk to Postgres, read files, clone Git repos, or return HTTP responses.

## adapters

`adapters` contains anything that talks to the outside world.

Examples:

- database access
- file system access
- object storage
- Git cloning
- external APIs
- queues

Adapters know how to do the outside-world work, but they should not own the app rules.

For example, a Postgres adapter can know how to update `current_translations`, but the service decides when that update needs to happen.

## routes

`routes` contains the HTTP routes.

Routes should be thin.

They should:

- read request data
- call a service
- return a response

They should not contain the main Fosslate logic.

For example, a route can call `approve_translation`, but it should not personally update approvals, current translations, and stats.

## services

`services` contains the actual Fosslate logic.

This is where the important app actions live:

- create a project
- add a namespace
- add or update strings
- add a translation
- vote on a translation
- approve a translation
- import strings
- export translations

Services decide what needs to happen. Adapters handle the outside-world details.

The main rule is that important writes should go through services. This keeps things like `current_translations`, `namespace_language_stats`, ratings, and approvals in sync.

## jobs

`jobs` contains background or repeated work.

Examples:

- cleanup deleted rows
- rebuild `current_translations`
- rebuild `namespace_language_stats`
- process queued imports
- run scheduled maintenance

Jobs should usually call services instead of duplicating app logic.

## General Rules

- Routes should not talk directly to the database.
- Models should not talk to the outside world.
- Adapters should not decide product behaviour.
- Services should be the main place for app rules.
- Jobs should reuse services where possible.
- Imports should use the same write paths as the UI.
- Exports should read from the fast read models where possible.

The short version:

```text
routes ask services what to do
services decide what should happen
adapters handle the outside world
models describe Fosslate things
jobs repeat service work in the background
```
