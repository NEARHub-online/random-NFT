use crate::*;

/// external contract calls

#[ext_contract(ext_contract)]
trait ExtContract {
    fn nft_mint_owner(
        &mut self,
        token_id: TokenId,
        receiver_id: ValidAccountId,
    );
}