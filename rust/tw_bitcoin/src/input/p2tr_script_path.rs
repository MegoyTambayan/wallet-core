use crate::{tweak_pubkey, Error, InputContext, Recipient, Result, TaprootScript};
use bitcoin::key::{PublicKey, TweakedPublicKey};
use bitcoin::script::ScriptBuf;
use bitcoin::taproot::{ControlBlock, TapNodeHash, TaprootSpendInfo};
use bitcoin::{OutPoint, PubkeyHash, Sequence, TxIn, Txid, WPubkeyHash, Witness};

#[derive(Debug, Clone)]
pub struct TxInputP2TRScriptPath {
    pub(crate) ctx: InputContext,
    pub(crate) recipient: Recipient<TaprootScript>,
    pub(crate) script: ScriptBuf,
    pub(crate) spend_info: TaprootSpendInfo,
}

impl TxInputP2TRScriptPath {
    pub fn new(
        txid: Txid,
        vout: u32,
        recipient: impl Into<Recipient<TaprootScript>>,
        satoshis: u64,
        script: ScriptBuf,
        spend_info: TaprootSpendInfo,
    ) -> Self {
        let recipient: Recipient<TaprootScript> = recipient.into();

        TxInputP2TRScriptPath {
            ctx: InputContext {
                previous_output: OutPoint { txid, vout },
                value: Some(satoshis),
                script_pubkey: ScriptBuf::new_v1_p2tr(
                    &secp256k1::Secp256k1::new(),
                    recipient.untweaked_pubkey(),
                    Some(recipient.merkle_root()),
                ),
                sequence: Sequence::default(),
                witness: Witness::new(),
            },
            recipient,
            script,
            spend_info,
        }
    }
    pub fn builder() -> TxInputP2TRScriptPathBuilder {
        TxInputP2TRScriptPathBuilder::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TxInputP2TRScriptPathBuilder {
    txid: Option<Txid>,
    vout: Option<u32>,
    recipient: Option<Recipient<TaprootScript>>,
    satoshis: Option<u64>,
    script: Option<ScriptBuf>,
    spend_info: Option<TaprootSpendInfo>,
}

impl TxInputP2TRScriptPathBuilder {
    pub fn new() -> TxInputP2TRScriptPathBuilder {
        TxInputP2TRScriptPathBuilder {
            txid: None,
            vout: None,
            recipient: None,
            satoshis: None,
            script: None,
            spend_info: None,
        }
    }
    pub fn txid(mut self, txid: Txid) -> TxInputP2TRScriptPathBuilder {
        self.txid = Some(txid);
        self
    }
    pub fn vout(mut self, vout: u32) -> TxInputP2TRScriptPathBuilder {
        self.vout = Some(vout);
        self
    }
    pub fn recipient(
        mut self,
        recipient: Recipient<TaprootScript>,
    ) -> TxInputP2TRScriptPathBuilder {
        self.recipient = Some(recipient);
        self
    }
    pub fn satoshis(mut self, satoshis: u64) -> TxInputP2TRScriptPathBuilder {
        self.satoshis = Some(satoshis);
        self
    }
    pub fn script(mut self, script: ScriptBuf) -> TxInputP2TRScriptPathBuilder {
        self.script = Some(script);
        self
    }
    pub fn spend_info(mut self, spend_info: TaprootSpendInfo) -> TxInputP2TRScriptPathBuilder {
        self.spend_info = Some(spend_info);
        self
    }
    pub fn build(self) -> Result<TxInputP2TRScriptPath> {
        Ok(TxInputP2TRScriptPath::new(
            self.txid.ok_or(Error::Todo)?,
            self.vout.ok_or(Error::Todo)?,
            self.recipient.ok_or(Error::Todo)?,
            self.satoshis.ok_or(Error::Todo)?,
            self.script.ok_or(Error::Todo)?,
            self.spend_info.ok_or(Error::Todo)?,
        ))
    }
}