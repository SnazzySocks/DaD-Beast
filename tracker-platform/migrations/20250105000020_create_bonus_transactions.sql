-- Create bonus_transactions table
-- Bonus point transaction ledger

CREATE TABLE bonus_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Transaction details
    transaction_type VARCHAR(50) NOT NULL, -- earned, spent, admin_adjust, transfer, gift
    amount DECIMAL(20,2) NOT NULL, -- Positive for earning, negative for spending
    balance_before DECIMAL(20,2) NOT NULL,
    balance_after DECIMAL(20,2) NOT NULL,

    -- Source/reason
    source_type VARCHAR(50), -- seeding, upload, request_fill, purchase, gift, admin
    source_id UUID, -- Related entity ID (torrent_id, request_id, etc.)
    description TEXT,

    -- Related entities
    torrent_id UUID REFERENCES torrents(id) ON DELETE SET NULL,
    bonus_rule_id UUID REFERENCES bonus_rules(id) ON DELETE SET NULL,
    related_user_id UUID REFERENCES users(id) ON DELETE SET NULL, -- For transfers/gifts

    -- Metadata
    metadata JSONB, -- Additional transaction data

    -- Admin actions
    admin_id UUID REFERENCES users(id) ON DELETE SET NULL,
    admin_notes TEXT,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_bonus_transactions_user_id ON bonus_transactions(user_id);
CREATE INDEX idx_bonus_transactions_transaction_type ON bonus_transactions(transaction_type);
CREATE INDEX idx_bonus_transactions_created_at ON bonus_transactions(created_at DESC);
CREATE INDEX idx_bonus_transactions_torrent_id ON bonus_transactions(torrent_id);
CREATE INDEX idx_bonus_transactions_bonus_rule_id ON bonus_transactions(bonus_rule_id);
CREATE INDEX idx_bonus_transactions_related_user_id ON bonus_transactions(related_user_id);
CREATE INDEX idx_bonus_transactions_admin_id ON bonus_transactions(admin_id);

-- Composite index for user transaction history
CREATE INDEX idx_bonus_transactions_user_created ON bonus_transactions(user_id, created_at DESC);

-- Index for transaction type analytics
CREATE INDEX idx_bonus_transactions_type_created ON bonus_transactions(transaction_type, created_at DESC);

COMMENT ON TABLE bonus_transactions IS 'Complete ledger of all bonus point transactions';
COMMENT ON COLUMN bonus_transactions.amount IS 'Transaction amount (positive = earned, negative = spent)';
COMMENT ON COLUMN bonus_transactions.source_type IS 'Source of transaction: seeding, upload, purchase, etc.';
COMMENT ON COLUMN bonus_transactions.source_id IS 'UUID of related entity (polymorphic)';
COMMENT ON COLUMN bonus_transactions.metadata IS 'Additional JSON data about the transaction';
