ALTER TABLE instance_setup
ADD COLUMN secrets_key TEXT;

UPDATE instance_setup
SET secrets_key = encode(gen_random_bytes(32), 'hex')
WHERE id = 1
  AND secrets_key IS NULL;

ALTER TABLE instance_setup
ALTER COLUMN secrets_key SET NOT NULL;
