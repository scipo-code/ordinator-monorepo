-- Test users for development
-- Password for all users: testpassword123

INSERT INTO users (id, email, password_hash, role, assets, provider, is_active)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'scheduler@test.com', '$argon2id$v=19$m=19456,t=2,p=1$l3P3RFoMkOlAocl5v+eJdw$KU8SpmAAqjcOXhTUO58WWyB2fQA6t+z0jMwOTplYDyE', 'Scheduler', '["DF", "test"]', 'Local', 1),
    ('22222222-2222-2222-2222-222222222222', 'supervisor@test.com', '$argon2id$v=19$m=19456,t=2,p=1$l3P3RFoMkOlAocl5v+eJdw$KU8SpmAAqjcOXhTUO58WWyB2fQA6t+z0jMwOTplYDyE', 'Supervisor', '["DF", "test"]', 'Local', 1),
    ('33333333-3333-3333-3333-333333333333', 'technician@test.com', '$argon2id$v=19$m=19456,t=2,p=1$l3P3RFoMkOlAocl5v+eJdw$KU8SpmAAqjcOXhTUO58WWyB2fQA6t+z0jMwOTplYDyE', 'Technician', '["DF", "test"]', 'Local', 1),
    ('44444444-4444-4444-4444-444444444444', 'admin@test.com', '$argon2id$v=19$m=19456,t=2,p=1$l3P3RFoMkOlAocl5v+eJdw$KU8SpmAAqjcOXhTUO58WWyB2fQA6t+z0jMwOTplYDyE', 'Admin', '["DF", "test"]', 'Local', 1);
