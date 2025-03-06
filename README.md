# NOVA Backend

## Project Structure
```bash
|--./
|---backend
|---nova-db
|---searcher
|---service
|---sei-client

```


1. **backend**
    NOVA web backend program, supports tls/wss, routing

1. **nova-db**
    NOVA's database system records user NFT, token holding data, stake, NFT token transaction data
1. **searcher**
    NOVA's data engine obtains real-time blockchain data, filters, parses, and calls nova db for storage

1. **service**
    NOVA's open services include http, grpc, rpc, and wss

## schedule
- [ ] *backend*
    - [x] NFT
    - [ ] TOKEN
- [x] *nova-db*
- [x] *searcher*
- [x] *sei-client*


# BACKEND API
### NFT APIS

1. Get the NFT held by the user
    > [GET]
    /user/nft/get_holding_nfts/**{wallet_address}**

  **response**

    ```json
        {
            [
                {
                    "name":"NFT Collect name",
                    "symbol":"NFT Collect symbol",
                    "creator":"NFT Collect creator",
                    "contract":"NFT Collect contract address",
                    "floor_price" :"NFT Collect floor price",
                    "nfts_hold":[
                        "name":"NFT name",
                        "key":"NFT key",
                        "token_id":"NFT token id",
                        "image":"NFT image",
                        "buy_price":"NFT buy price",
                        "market_fee":"buy NFT pay market fee",
                        "floor_price":"NFT floor price",
                        "gas_fee":[
                            "demon":"usei",
                            "amount":"100000"
                        ],
                        "unrealized_gains":"NFT unrealized gains",
                        "attributes":[
                            "trait_type":"String",
                            "value":"Sring"
                        ],
                        "ts":"buy time",
                        "tx_hash":"buy tx_hash"

                    ]
                }
            ]
        }
    ```
    ---
    - name 
    *type* : String
    - symbol
    *type* : String
    - creator
    *type* : String
    - contract
    *type* : String
    - floor_price
    *type* : String | null
    - nfts_hold
    *type* : List
        
        - name
        *type* : String
        - key
        *type* : String
        - token_id 
        *type* : String
        - image 
        *type* : String
        - buy_price 
        *type* : String | null
        - market_fee
        *type* : String | null
        - floor_price
        *type* : String | null
        - gas_fee
        *type* : List
            - demon 
            *type* : String
            - amount
            *type* : String

         - unrealized_gains
         *type* : String | null
         - attributes
         *type* : List
            - trait_type
            *type* : String
            - value
            *type* : String

         - ts
         *type* : String | null
         - tx_hash
         *type* : String | null


    ---
 


1. Get user income NFT
    >[GET]
    /user/nft/get_income_nfts/**{wallet_address}**

    **response**
    ```json
    {
        [
            {
                "name":"NFT Collect name",
                "creator":"NFT Collect creator",
                "contract":"NFT Collect contract address",
                "income_nfts":[
                    "name":"NFT name",
                    "key":"NFT key", 
                    "token_id":"NFT token id":,
                    "image":"NFT image url",
                    "buy_price":"NFT buy price",
                    "sell_price":"NFT sell price",
                    "hold_time":"nft holding time",
                    "realized_gains":"nft realized gains",
                    "paid_fee":"buy and sell nft use all paid fee",
                ]
            }
        ]
    }
    ```

    ---
    - name 
    *type* : String
    - symbol
    *type* : String
    - creator
    *type* : String
    - contract
    *type* : String
    - income_nfts
    *type* : List
        - name
        *type* : String
        - key 
        *type* : String
        - token_id
        *type* : String
        - image
        *type* : String
        - buy_price
        *type* : String
        - sell_price
        *type* : String
        - hold_time
        *type* : String
        - realized_gains
        *type* : String
        - paid_fee
        *type* : String

    ---

1. Get the top data of NFT held by the user
    > [GET]
    /user/nft/get_income_nfts/**{wallet_address}**

    **response**

    ```json
        {
            "top_gainers":[
                "name":"NFT name", 
                "key":"NFT key",
                "token_id":"NFT id",
                "image":"NFT image url",
                "buy_price":Option<String>,
                "market_fee":Option<String>,
                "floor_price":Option<String>,
                "gas_fee":Vec<FeeAmount>,
                "unrealized_gains":String,
                "attributes":Vec<NftAttribute>,
                "ts":Option<String>,
                "tx_hash":Option<String>,
            ],
            "top_losser":[],
        }
    ```

    ---


