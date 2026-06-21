use sqlx::PgPool;

use crate::adapters::{postgres::PostgresAdapter, resend::EmailDeliveryClient};

pub mod approvals;
pub mod languages;
pub mod maintenance;
pub mod namespaces;
pub mod projects;
pub mod setup;
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
pub use setup::SetupService;
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
    pub setup: SetupService,
    pub translations: TranslationService,
    pub votes: VoteService,
    pub approvals: ApprovalService,
}

impl Services {
    pub fn with_setup(
        db: PgPool,
        setup_secret: String,
        public_app_url: String,
        secrets_key: String,
        email_delivery: EmailDeliveryClient,
    ) -> Self {
        let postgres = PostgresAdapter::new(db);

        Self {
            users: UserService::new(postgres.clone()),
            projects: ProjectService::new(postgres.clone()),
            languages: LanguageService::new(postgres.clone()),
            maintenance: MaintenanceService::new(postgres.clone()),
            namespaces: NamespaceService::new(postgres.clone()),
            source_strings: SourceStringService::new(postgres.clone()),
            stats: StatsService::new(postgres.clone()),
            setup: SetupService::new(
                postgres.clone(),
                email_delivery,
                setup_secret,
                public_app_url,
                secrets_key,
            ),
            translations: TranslationService::new(postgres.clone()),
            votes: VoteService::new(postgres.clone()),
            approvals: ApprovalService::new(postgres),
        }
    }
}
