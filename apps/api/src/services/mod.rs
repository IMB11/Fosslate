use sqlx::PgPool;

use crate::adapters::postgres::PostgresAdapter;

pub mod approvals;
pub mod languages;
pub mod maintenance;
pub mod namespaces;
pub mod projects;
pub mod source_strings;
pub mod stats;
pub mod translations;
pub mod users;
pub mod votes;

pub use approvals::ApprovalService;
pub use languages::LanguageService;
pub use maintenance::MaintenanceService;
pub use namespaces::NamespaceService;
pub use projects::ProjectService;
pub use source_strings::SourceStringService;
pub use stats::StatsService;
pub use translations::TranslationService;
pub use users::UserService;
pub use votes::VoteService;

#[derive(Clone)]
pub struct Services {
    pub users: UserService,
    pub projects: ProjectService,
    pub languages: LanguageService,
    #[allow(dead_code)]
    pub maintenance: MaintenanceService,
    pub namespaces: NamespaceService,
    pub source_strings: SourceStringService,
    pub stats: StatsService,
    pub translations: TranslationService,
    pub votes: VoteService,
    pub approvals: ApprovalService,
}

impl Services {
    pub fn new(db: PgPool) -> Self {
        let postgres = PostgresAdapter::new(db);

        Self {
            users: UserService::new(postgres.clone()),
            projects: ProjectService::new(postgres.clone()),
            languages: LanguageService::new(postgres.clone()),
            maintenance: MaintenanceService::new(postgres.clone()),
            namespaces: NamespaceService::new(postgres.clone()),
            source_strings: SourceStringService::new(postgres.clone()),
            stats: StatsService::new(postgres.clone()),
            translations: TranslationService::new(postgres.clone()),
            votes: VoteService::new(postgres.clone()),
            approvals: ApprovalService::new(postgres),
        }
    }
}
