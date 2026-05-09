use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuantityType {
    #[serde(rename = "current")] Current,
    #[serde(rename = "traded")] Traded,
    #[serde(rename = "safe_keeping")] SafeKeeping,
    #[serde(rename = "segregated")] Segregated,
    #[serde(rename = "frozen")] Frozen,
    #[serde(rename = "available")] Available,
    #[serde(rename = "pending_settlement")] PendingSettlement,
    #[serde(rename = "pledged")] Pledged,
    #[serde(rename = "recalled")] Recalled,
    #[serde(rename = "accrued_interest")] AccruedInterest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PriceType {
    #[serde(rename = "clean")] Clean,
    #[serde(rename = "dirty")] Dirty,
    #[serde(rename = "market")] Market,
    #[serde(rename = "strike")] Strike,
    #[serde(rename = "forward")] Forward,
    #[serde(rename = "all_in")] AllIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmountType {
    #[serde(rename = "gross")] Gross,
    #[serde(rename = "net")] Net,
    #[serde(rename = "settlement")] Settlement,
    #[serde(rename = "commission")] Commission,
    #[serde(rename = "tax")] Tax,
    #[serde(rename = "fee")] Fee,
    #[serde(rename = "accrued_interest")] AccruedInterest,
    #[serde(rename = "stamp_duty")] StampDuty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DateType {
    #[serde(rename = "trade")] Trade,
    #[serde(rename = "settlement")] Settlement,
    #[serde(rename = "value")] Value,
    #[serde(rename = "ex")] Ex,
    #[serde(rename = "maturity")] Maturity,
    #[serde(rename = "payment")] Payment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    #[serde(rename = "DEBIT")] Debit,
    #[serde(rename = "CREDIT")] Credit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    #[serde(rename = "SUBLEDGER")] Subledger,
    #[serde(rename = "GL")] GL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubledgerType {
    #[serde(rename = "TRADING")] Trading,
    #[serde(rename = "CASH")] Cash,
    #[serde(rename = "PNL")] PNL,
    #[serde(rename = "SETTLEMENT")] Settlement,
    #[serde(rename = "POSITION")] Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    #[serde(rename = "ASSET")] Asset,
    #[serde(rename = "LIABILITY")] Liability,
    #[serde(rename = "EQUITY")] Equity,
    #[serde(rename = "INCOME")] Income,
    #[serde(rename = "EXPENSE")] Expense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    #[serde(rename = "submitted")] Submitted,
    #[serde(rename = "enriched")] Enriched,
    #[serde(rename = "validated")] Validated,
    #[serde(rename = "posted")] Posted,
    #[serde(rename = "failed")] Failed,
    #[serde(rename = "cancelled")] Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    #[serde(rename = "ACCRUED_INTEREST")] AccruedInterest,
    #[serde(rename = "COMMISSION")] Commission,
    #[serde(rename = "TAX")] Tax,
    #[serde(rename = "SETTLEMENT")] Settlement,
    #[serde(rename = "QUANTITY_TRANSFER")] QuantityTransfer,
    #[serde(rename = "CASH_MOVEMENT")] CashMovement,
    #[serde(rename = "CORP_ACTION")] CorpAction,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Side::Debit => write!(f, "DEBIT"), Side::Credit => write!(f, "CREDIT") }
    }
}

impl TryFrom<&str> for Side {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s { "DEBIT" => Ok(Side::Debit), "CREDIT" => Ok(Side::Credit), _ => Err(format!("invalid side: {}", s)) }
    }
}
