use crate::*;

// One can provide a name, e.g. `ext` to use for generated methods.
#[ext_contract(ext)]
pub trait ExtCrossWhitelist {
    fn on_get_whitelist(&self, #[callback_unwrap] quantity: U128) -> U128;
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        title: String,
        url: String,
        size: u8,
        private: bool,
        description: String
    ){
        let mint_price = self.get_nft_price(size, private);
        assert!(
            env::attached_deposit() >= mint_price,
            "Attached deposit must be greater than or equal to MINT_PRICE"
        );
        
        let extra = format!("{{'room_size': {}, 'private_room': {}}}", size.to_string(), (private).to_string());

        let _metadata = TokenMetadata {
            title: Some(title.into()),
            description: Some(description.into()),
            media: Some(url.to_string()),
            media_hash: None,
            copies: Some(420u64),
            issued_at: Some(env::block_timestamp()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: Some(extra.into()),
            reference: None,
            reference_hash: None,
        };
        self.token_minted += 1;

        let mut royalty = HashMap::new();

        //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
        assert!(self.perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

        //iterate through the perpetual royalties and insert the account and amount in the royalty map
        for (account, amount) in self.perpetual_royalties.to_vec() {
            royalty.insert(account, amount);
        }

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&self.token_minted.to_string(), &token).is_none(),
            "Token already exists"
        );

        //if the room is private, add it to tickets map
        if private {
            self.room_tickets.insert(&self.token_minted.to_string(), &0);
        } 

        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&self.token_minted.to_string(), &_metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &self.token_minted.to_string());

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![self.token_minted.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        // Transfert amout to receiver
        // Promise::new(self.receiver_id.clone().into()).transfer(MINT_PRICE - 10_000_000_000_000_000_000_000);
    }

    #[private]
    fn get_nft_price(
        &self,
        size: u8,
        private: bool
    ) -> u128 {
        let mut price: u128 = 7_000_000_000_000_000_000_000_000;
        match size{
            1=>price+=0,
            2=>price+=0,
            3=>price+=0,
            5=>price+=2_000_000_000_000_000_000_000_000,
            10=>price+=7_000_000_000_000_000_000_000_000,
            15=>price+=10_000_000_000_000_000_000_000_000,
            20=>price+=20_000_000_000_000_000_000_000_000,
            30=>price+=50_000_000_000_000_000_000_000_000,
            40=>price+=150_000_000_000_000_000_000_000_000,
            50=>price+=300_000_000_000_000_000_000_000_000,
            _=>price+=u128::from(size)*10_000_000_000_000_000_000_000_000,   
        }
        if private {
            price+=20_000_000_000_000_000_000_000_000;
        }
        price
    }
}