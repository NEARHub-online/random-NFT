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
    ) -> Promise {
        assert!(
            env::attached_deposit() >= MINT_PRICE,
            "Attached deposit must be greater than MINT_PRICE"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        assert!(
            self.token_minted_users < MAX_NFT_MINT_USERS,
            "Max token on sale is MAX_NFT_MINT_USERS"
        );

        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - GAS_RESERVED_FOR_CURRENT_CALL;
        Promise::new(env::current_account_id()).function_call(
            "nft_mint_owner".to_string(),
            json!({ "receiver_id": receiver_id.to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            75_000_000_000_000_000_000_000,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
    }


    #[payable]
    pub fn nft_mint_owner(
        &mut self,
        receiver_id: AccountId,
    ) {
        assert!(
            env::is_valid_account_id(receiver_id.as_bytes()),
            "The receiver account ID is invalid"
        );
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only the contract owner can call this method"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        let url = format!("{}{}.png", NFT_IMAGES, (self.token_minted + 1).to_string());
        let reference = format!("{}{}.json", NFT_JSON, (self.token_minted + 1).to_string());
        let title: String =format!("NEARHUB Comics PFP HRMS #0 #{}", (self.token_minted + 1).to_string());
        let _metadata = TokenMetadata {
            title: Some(title.into()),
            description: Some("2100 PFP NFTs created in comic book cover style to help launch NEARHUB comics very first issue that’ll feature the origin story of HRMS and how he begins his adventure through the NEARverse. Each page of the comic will be created based on a monthly contest that anyone can participate in, in order to help build the story. Holders of this NFT will be able to participate in the monthly contest at a discounted price, earn future staking rewards, &amp; more. Visit comics.nearhub.club for more details.".into()),
            media: Some(url.to_string()),
            media_hash: None,
            copies: Some(1u64),
            issued_at: Some(env::block_timestamp()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: Some(reference.to_string()),
            reference_hash: None,
        };
        self.token_minted += 1;
        if env::current_account_id() != env::signer_account_id() {
            self.token_minted_users += 1;
        }

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
        Promise::new(self.receiver_id.clone().into()).transfer(NH_FEE);
        Promise::new(self.receiver1_id.clone().into()).transfer(NC_FEE);
    }

    #[payable]
    pub fn get_free_token(
        &mut self,
    ) -> Promise {
        assert!(
            env::attached_deposit() == 0,
            "Attached deposit must be 0 for a free NFT"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        assert!(
            self.token_minted_users < MAX_NFT_MINT_USERS,
            "Max token on sale is MAX_NFT_MINT_USERS"
        );
        assert!(
            self.nft_supply_for_owner(env::signer_account_id()) == U128(2),
            "You should have exactly 2 NFT to get a free one" 
        );

        // Get external contract whitelist
        // let amount: PromiseOrValue<U128> = Promise::new(env::current_account_id()).function_call(
        //     "nft_supply_for_owner".to_string(),
        //     json!({ "receiver_id": env::signer_account_id().to_string() }) // method arguments
        //         .to_string()
        //         .into_bytes(),
        //     0,    // amount of yoctoNEAR to attach
        //     Gas(0)).then(ext::on_get_whitelist(env::current_account_id(), 0, GAS_RESERVED_FOR_CURRENT_CALL)).into();

        // assert!(
        //     amount.into() != U128(0),
        //     "You are not in the whitelist"
        // );

        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - GAS_RESERVED_FOR_CURRENT_CALL;
        Promise::new(env::current_account_id()).function_call(
            "nft_mint_owner".to_string(),
            json!({ "receiver_id": env::signer_account_id().to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            0,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
        
    }

    #[private]
    pub fn on_get_whitelist(&self, #[callback_unwrap] quantity: U128) -> U128 {
        quantity
    }

}