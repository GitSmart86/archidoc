# Architecture (AI Context)

folder_scaffold/ — Scaffold folder templates — copies named template trees with `{{variable}}` substitution.
init_cmd/ — Init command — generates files from environment context.
api/ — REST API gateway — handles authentication and request routing.
database/ — Persistence layer — manages database connections and queries.
events/ — Domain event bus — decouples producers from consumers.

examples.rust-example.src.api -> database: "Persists user data" (sqlx)
examples.rust-example.src.api -> events: "Publishes domain events" (channel)
