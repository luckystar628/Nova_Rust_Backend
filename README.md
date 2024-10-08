# NOVA 后端

## 项目结构
```bash
|--./
|---backend
|---nova-db
|---searcher
|---service
|---sei-client

```


1. **backend**
    NOVA web后端程序 ，支持tls/wss，路由

1. **nova-db**
    NOVA 的数据库系统，记录用户nft，token持有数据，stake ，nft token 交易数据

1. **searcher**
    NOVA的数据引擎，获取实时区块链数据，过滤，解析，调用nova db，进行储存

1. **service**
    NOVA 对外开放的服务，http、grpc、rpc、wss

## 进度
- [ ] *backend*
    - [x] NFT
    - [ ] TOKEN
- [x] *nova-db*
- [x] *searcher*
- [x] *sei-client*


# BACKEND API
### NFT APIS

1. 获取用户持有的NFT
    > [GET]
    /user/nft/get_holding_nfts/**{wallet_address}**

    **响应**
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
 


1. 获取用户income NFT
    >[GET]
    /user/nft/get_income_nfts/**{wallet_address}**

    **响应**
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

1. 获取用户持有NFT的TOP数据
    > [GET]
    /user/nft/get_income_nfts/**{wallet_address}**

    **响应**

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


