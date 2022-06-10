Non-fungible Token (NFT)
===================

export NEAR_ENV=testnet
near create-account mmanice.nft.nearhubonline.testnet --masterAccount nft.nearhubonline.testnet --initialBalance 4
near deploy --account_id mmanice.nft.nearhubonline.testnet --wasm-file "out/non_fungible_token (50).wasm"
near call mmanice.nft.nearhubonline.testnet new_default_meta '{"owner_id": "mmanice.nft.nearhubonline.testnet", "receiver_id": "balda.testnet", "receiver1_id": "balda.testnet", "receiver2_id": "balda.testnet"}' --account-id mmanice.nft.nearhubonline.testnet

near call mmanice.nft.nearhubonline.testnet nft_mint '{}' --account-id  balda.testnet --deposit 3 --gas=75000000000000


## Mainnet
near create-account mmanice.nft.nearhubonline.near --masterAccount nft.nearhubonline.near --initialBalance 4
near deploy --account_id mmanice.nft.nearhubonline.near --wasm-file "out/non_fungible_token (50).wasm"
near call mmanice.nft.nearhubonline.near new_default_meta '{"owner_id": "mmanice.nft.nearhubonline.near", "receiver_id": "nearhub-dao.sputnik-dao.near", "receiver1_id": "ifeelvirtuel.near", "receiver2_id": "nearhub-dao.sputnik-dao.near"}' --account-id mmanice.nft.nearhubonline.near


To show it on paras
near call paras-marketplace-v2.testnet storage_deposit '{"accountId":"balda.testnet"}' --accountId balda.testnet --depositYocto 8590000000000000000000
near call random-nft-4.nearhubonline.testnet nft_approve '{"token_id":"1","account_id":"paras-marketplace-v1.testnet","msg":"{\"market_type\":\"sale\",\"price\":\"1000000000000000000000000\",\"ft_token_id\":\"near\"}"}' --accountId balda.testnet --depositYocto 400000000000000000000

near call --accountId random-nft-2.nearhubonline.testnet paras-token-v2.testnet nft_approve '{"token_id":"1","account_id":"paras-marketplace-v2.testnet","msg":"{\"market_type\":\"sale\",\"price\":\"1000000000000000000000000\",\"ft_token_id\":\"near\"}"}' --depositYocto 400000000000000000000