pub const QX_ADDRESS: &str = "BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID";

pub const QX_CONTRACT_INDEX: u32 = 1;

#[derive(Debug)]
pub enum QxFunctions {
    QxGetFee = 1,
    QxGetAssetAskOrder = 2,
    QxGetAssetBidOrder = 3,
    QxGetEntityAskOrder = 4,
    QxGetEntityBidOrder = 5
}

impl QxFunctions {
    pub fn from_u16(b: u16) -> Result<Self, String> {
        match b {
            1 => Ok(QxFunctions::QxGetFee),
            2 => Ok(QxFunctions::QxGetAssetAskOrder),
            3 => Ok(QxFunctions::QxGetAssetBidOrder),
            4 => Ok(QxFunctions::QxGetEntityAskOrder),
            5 => Ok(QxFunctions::QxGetEntityBidOrder),
            _ => Err("Invalid value".to_string())
        }
    }
}

#[derive(Debug)]
pub enum QxProcedure {
    QxIssueAsset = 1,
    QxTransferShare = 2,
    QxPlaceholder0 = 3,
    QxPlaceholder1 = 4,
    QxAddAskOrder = 5,
    QxAddBidOrder = 6,
    QxRemoveAskOrder = 7,
    QxRemoveBidOrder = 8,
    QxTransferShareManagementRights = 9
}

pub mod asset_transfer;
pub mod order;
pub mod orderbook;
pub mod asset;
