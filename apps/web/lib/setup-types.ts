export type SetupStep = "github" | "gitlab" | "email" | "complete";

export type AuthProviderSetupStatus = {
  enabled: boolean;
  configured: boolean;
  skipped: boolean;
  has_client_secret: boolean;
  client_id: string | null;
  base_url: string | null;
  scopes: string[];
  callback_url: string;
  configured_at: string | null;
  skipped_at: string | null;
};

export type EmailDeliverySetupStatus = {
  configured: boolean;
  has_api_key: boolean;
  provider: string | null;
  from_name: string | null;
  from_email: string | null;
  last_test_recipient: string | null;
  last_tested_at: string | null;
  last_test_message_id: string | null;
};

export type SetupStatus = {
  required: boolean;
  completed: boolean;
  next_step?: SetupStep | null;
  github: AuthProviderSetupStatus;
  gitlab: AuthProviderSetupStatus;
  email: EmailDeliverySetupStatus;
};

export type SaveSsoProviderRequest =
  | {
      enabled: false;
    }
  | {
      enabled: true;
      client_id: string;
      client_secret: string;
      base_url?: string | null;
    };

export type TestEmailDeliveryRequest = {
  resend_api_key: string;
  from_name: string;
  from_email: string;
  test_recipient: string;
};

export type SetupCompleteResponse = {
  next: string;
};

