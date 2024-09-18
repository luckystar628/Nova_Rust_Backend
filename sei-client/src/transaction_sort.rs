use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{data_feild_structs::{nft_data_struct, stake_data_sturct, token_data_struct}, data_rp_structs::{self, tx_rp_struct::{self, FeeAmount}}};


#[derive(Debug,Clone,Deserialize,Serialize)]
pub enum TransactionEvent {
    NftMint(Vec<nft_data_struct::Mint>),
    NftBatchBids(Vec<nft_data_struct::BatchBids>),
    NftOnlyTransfer(Vec<nft_data_struct::Transfer>),
    NftCreateAuction(Vec<nft_data_struct::CreateAuction>),
    NftCancelAuction(Vec<nft_data_struct::CancelAuction>),
    NftPurchaseCart(Vec<nft_data_struct::PurchaseCart>),
    NftAcceptBid(Vec<nft_data_struct::AcceptBid>),
    NftFixedSell(Vec<nft_data_struct::FixedSell>),
    NftOnlyCreateAuction(Vec<nft_data_struct::OnlyCreateAuction>),

    TokenHeihtSwap(Vec<token_data_struct::TokenSwap>),
    TokenNormalSwap(Vec<token_data_struct::TokenSwap>),

    TokenTransferByWei(Vec<token_data_struct::TokenTransfer>),
    TokenTransferByBank(Vec<token_data_struct::TokenTransfer>),
    TokenTransferByContract(Vec<token_data_struct::ContractTokenTransfer>),


    Delegate(Vec<stake_data_sturct::Stake>),
    Undelegate(Vec<stake_data_sturct::Stake>),
    Unkonw,
}


pub trait Transaction {
    
    type TransactionSender;
    type TransactionFee;
    type Ts;
    type Tx;


    fn transaction_event_type(
        &self,
        transaction_sender:Self::TransactionSender,
        transaction_fee:Self::TransactionFee,
        ts:Self::Ts,
        tx:Self::Tx)-> TransactionEvent;

    fn is_nft_mint(&self) -> bool;
    fn is_nft_batch_bids(&self)->bool;
    fn is_nft_fixed_sell(&self)->bool;
    fn is_nft_only_transfer(&self)->bool;
    fn is_nft_create_auction(&self)->bool;
    fn is_nft_cancel_auction(&self)->bool;
    fn is_nft_purchase_nft(&self)->bool;
    fn is_nft_only_create_auction(&self)->bool;
    fn is_nft_accept_bid(&self)->bool;


    fn is_token_normal_swap(&self) -> bool;
    fn is_token_height_swap(&self) -> bool;

    fn is_evm_token_transfer_by_wei(&self)->bool;
    fn is_token_transfer_by_bank(&self)->bool;
    fn is_token_transfer_by_contract(&self)->bool;
    
    fn is_delegate(&self)->bool;
    fn is_undelegate(&self)->bool;

    
}

impl Transaction for tx_rp_struct::Log {
    type TransactionSender =Option<String>;
    type TransactionFee=Vec<FeeAmount>;
    type Ts = String;
    type Tx = String;


    fn transaction_event_type(&self,transaction_sender:Self::TransactionSender,transaction_fee:Self::TransactionFee,ts:Self::Ts,tx:Self::Tx)->TransactionEvent {
        
        if self.is_nft_only_create_auction(){
            let mut x=vec![];
            self.events.iter().for_each(|event|{
                if event._type=="wasm-create_auction"{

                
                    event.attributes.iter().enumerate().for_each(|(index,attr)|{
                        if attr.key=="token_id"{
                            x.push(
                                nft_data_struct::OnlyCreateAuction{
                                    collection:event.attributes[index-1].value.to_owned(),
                                    nft_id:attr.value.to_owned(),
                                    auction_price:event.attributes[index+3].value.to_owned(),
                                    transaction_sender:transaction_sender.to_owned(),
                                    fee:transaction_fee.to_owned(),
                                    ts:ts.to_owned(),
                                    tx:tx.to_owned(),
                                }
                            )
                        }
                    });
                }
            });

            TransactionEvent::NftOnlyCreateAuction(x)
        
        }else if self.is_nft_batch_bids() {

        
            let mut x=vec![];
            let get_nft_inds_and_indexs=|events:&Vec<data_rp_structs::tx_rp_struct::Event>|->HashMap<String,usize>{
                    
                let mut nft_id_and_index:HashMap<String,usize>=HashMap::new();
                events.iter().find(|event|{event._type=="wasm"}).and_then(|event|{
                    Some(
                        event.attributes.iter().enumerate().for_each(|(index,attr)|{
                            if attr.key=="token_id"{
                                nft_id_and_index.insert(attr.value.to_owned(), index.to_owned());
                            }
                        })
                    )
                });
                nft_id_and_index
            };

            let wasm_event=self.events.iter().find(|event| event._type=="wasm").unwrap();
            let wasm_buy_now_event=self.events.iter().find(|event| event._type=="wasm-buy_now").unwrap();

            let nft_ids_and_indes=get_nft_inds_and_indexs(&self.events);
            

            for (nft_id,wasm_nft_index) in nft_ids_and_indes{
                wasm_buy_now_event.attributes.iter().enumerate().for_each(|(wasm_buy_now_index,attr)|{
                    if attr.key=="nft_token_id" && attr.value==nft_id{
                        x.push(
                            nft_data_struct::BatchBids{
                                collection:wasm_event.attributes[wasm_nft_index-4].value.to_owned(),
                                sender:wasm_event.attributes[wasm_nft_index-2].value.to_owned(),
                                recipient:wasm_event.attributes[wasm_nft_index-1].value.to_owned(),
                                nft_id:nft_id.to_owned(),
                                sale_price:wasm_buy_now_event.attributes[wasm_buy_now_index-2].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee:transaction_fee.to_owned(),
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                });
            }
            
            TransactionEvent::NftBatchBids(x)
        
        }else if self.is_nft_fixed_sell() {

            let mut x=vec![];
            self.events.iter().for_each(|event|{
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="action" && attr.value=="fixed_sell"{
                        x.push(
                            nft_data_struct::FixedSell{
                                collection:event.attributes[index+7].value.to_owned(),
                                sender:event.attributes[index+9].value.to_owned(),
                                recipient:event.attributes[index+10].value.to_owned(),
                                nft_id:event.attributes[index+3].value.to_owned(),
                                sale_price:event.attributes[index+4].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee:transaction_fee.to_owned(),
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                })
            });
            TransactionEvent::NftFixedSell(x)
        
        }else if self.is_nft_accept_bid() {

            let mut x:Vec<nft_data_struct::AcceptBid>=vec![];

            let get_nft_ids_and_indexs=|event:&tx_rp_struct::Event|->HashMap<String,usize>{
                let mut nft_ids_and_indexs:HashMap<String,usize>=HashMap::new();
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id"{
                        nft_ids_and_indexs.insert(attr.value.to_owned(), index.to_owned());
                    }
                });
                nft_ids_and_indexs
            };

            let wasm_event=self.events.iter().find(|event|{event._type=="wasm"}).unwrap();
            let wasm_accept_bid_event=self.events.iter().find(|event|{event._type=="wasm-accept_bid"}).unwrap();

            let nft_ids_and_indexs=get_nft_ids_and_indexs(wasm_event);
            for (nft_id,nft_id_index) in nft_ids_and_indexs{
                wasm_accept_bid_event.attributes.iter().enumerate().for_each(|( index,attr)|{
                    if attr.key=="token_id" && attr.value == nft_id{
                        x.push(
                            nft_data_struct::AcceptBid{
                                collection: wasm_event.attributes[nft_id_index-4].value.to_owned(), 
                                sender:wasm_event.attributes[nft_id_index-2].value.to_owned(), 
                                recipient:wasm_event.attributes[nft_id_index-1].value.to_owned(), 
                                nft_id:nft_id.to_owned(),
                                bidder: wasm_accept_bid_event.attributes[index+3].value.to_owned(), 
                                seller: wasm_accept_bid_event.attributes[index+2].value.to_owned(), 
                                sale_price:wasm_accept_bid_event.attributes[index+1].value.to_owned(), 
                                marketplace_fee:wasm_accept_bid_event.attributes[index+8].value.to_owned(), 
                                royalties:wasm_accept_bid_event.attributes[index+12].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee: transaction_fee.to_owned(), 
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                })
            }

            
            TransactionEvent::NftAcceptBid(x)
        
        }else if self.is_nft_create_auction() {

            let mut x:Vec<nft_data_struct::CreateAuction>=vec![];

            let get_nft_ids_and_indexs=|event:&tx_rp_struct::Event|->HashMap<String,usize>{
                let mut nft_ids_and_indexs:HashMap<String,usize>=HashMap::new();
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id"{
                        nft_ids_and_indexs.insert(attr.value.to_owned(), index.to_owned());
                    }
                });
                nft_ids_and_indexs
            };

            let wasm_event=self.events.iter().find(|event|{event._type=="wasm"}).unwrap();
            let wasm_create_auction_event=self.events.iter().find(|event|{event._type=="wasm-create_auction"}).unwrap();
            
            let nft_ids_and_indexs=get_nft_ids_and_indexs(wasm_event);
            for (nft_id,nft_id_index) in nft_ids_and_indexs{
                wasm_create_auction_event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id" && attr.value==nft_id{
                        x.push(
                            nft_data_struct::CreateAuction{
                                collection:wasm_event.attributes[nft_id_index-4].value.to_owned(),
                                sender:wasm_event.attributes[nft_id_index-2].value.to_owned(),
                                recipient:wasm_event.attributes[nft_id_index-1].value.to_owned(),
                                nft_id:nft_id.to_owned(),
                                auction_price:wasm_create_auction_event.attributes[index+3].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee: transaction_fee.to_owned(), 
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                })
            }


            TransactionEvent::NftCreateAuction(x)
        
        }else if self.is_nft_cancel_auction() {

            let mut x:Vec<nft_data_struct::CancelAuction>=vec![];

            let get_nft_ids_and_indexs=|event:&tx_rp_struct::Event|->HashMap<String,usize>{
                let mut nft_ids_and_indexs:HashMap<String,usize>=HashMap::new();
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id"{
                        nft_ids_and_indexs.insert(attr.value.to_owned(), index.to_owned());
                    }
                });
                nft_ids_and_indexs
            };

            let wasm_event=self.events.iter().find(|event|{event._type=="wasm"}).unwrap();
            let wasm_cancel_auction_event=self.events.iter().find(|event|{event._type=="wasm-cancel_auction"}).unwrap();

            let nft_ids_and_indexs=get_nft_ids_and_indexs(wasm_event);
            for (nft_id,nft_id_index) in nft_ids_and_indexs{
                wasm_cancel_auction_event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id" && attr.value==nft_id{
                        x.push(
                            nft_data_struct::CancelAuction{
                                collection:wasm_event.attributes[nft_id_index-4].value.to_owned(),
                                sender:wasm_event.attributes[nft_id_index-2].value.to_owned(),
                                recipient:wasm_event.attributes[nft_id_index-1].value.to_owned(),
                                nft_id:nft_id.to_owned(),
                                auction_price:wasm_cancel_auction_event.attributes[index+2].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee: transaction_fee.to_owned(), 
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                })
            }
            TransactionEvent::NftCancelAuction(x)
        
        }else if self.is_nft_mint(){
            
            let mut x=vec![];

            let add_index=|attrs:&Vec<data_rp_structs::tx_rp_struct::Attribute>|->Vec<usize>{
                let mut mint_indexs=vec![];
                attrs.iter().enumerate().for_each(|(index,attr)|{
                    if attr.value=="mint"{
                        mint_indexs.push(index)
                    }
                });
                mint_indexs
            };

            self.events.iter().for_each(|event|{

                if event._type=="wasm"{
                    let mint_indexs= add_index(&event.attributes);
                    if event.attributes.len()>=12{
                        for mint_index in mint_indexs{
                            //  is mint_nft
                            if event.attributes[mint_index-7].key == "action" &&event.attributes[mint_index-7].value=="mint_nft"{
                                x.push(                            
                                    nft_data_struct::Mint{
                                        collection:event.attributes[mint_index-1].value.to_owned(),
                                        recipient:event.attributes[mint_index-4].value.to_owned(),
                                        nft_id:event.attributes[mint_index+3].value.to_owned(),
                                        price:Some(event.attributes[mint_index-2].value.to_owned()),
                                        transaction_sender:transaction_sender.to_owned(),
                                        fee:transaction_fee.to_owned(),
                                        ts:ts.to_owned(),
                                        tx:tx.to_owned()
                                    }
                                )
                            }else {
                                // is not mint_nft , just mint
                                x.push(                            
                                    nft_data_struct::Mint{
                                        collection:event.attributes[mint_index-1].value.to_owned(),
                                        recipient:event.attributes[mint_index+2].value.to_owned(),
                                        nft_id:event.attributes[mint_index+3].value.to_owned(),
                                        price:None,
                                        transaction_sender:transaction_sender.to_owned(),
                                        fee:transaction_fee.to_owned(),
                                        ts:ts.to_owned(),
                                        tx:tx.to_owned()
                                    }
                                )
                            }                      
                        }
                    }else {
                        for mint_index in mint_indexs{
                            x.push(                            
                                nft_data_struct::Mint{
                                    collection:event.attributes[mint_index-1].value.to_owned(),
                                    recipient:event.attributes[mint_index+2].value.to_owned(),
                                    nft_id:event.attributes[mint_index+3].value.to_owned(),
                                    price:None,
                                    transaction_sender:transaction_sender.to_owned(),
                                    fee:transaction_fee.to_owned(),
                                    ts:ts.to_owned(),
                                    tx:tx.to_owned()
                                }
                            )
                        }
                    }
                }
            });
            TransactionEvent::NftMint(x)

        }else if self.is_nft_only_transfer() {
            
            let mut x=vec![];

            self.events.iter().for_each(|event|{
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="action" && attr.value=="transfer_nft"{
                        x.push(
                            nft_data_struct::Transfer{
                                collection:event.attributes[index-1].value.to_owned(),
                                sender:event.attributes[index+1].value.to_owned(),
                                recipient:event.attributes[index+2].value.to_owned(),
                                nft_id:event.attributes[index+3].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee:transaction_fee.to_owned(),
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                })
            });
            TransactionEvent::NftOnlyTransfer(x)

        }else if self.is_nft_purchase_nft() {

            let mut x:Vec<nft_data_struct::PurchaseCart>=vec![];

            let wasm_event=self.events.iter().find(|event| event._type=="wasm").unwrap();
            let wasm_buy_now_event=self.events.iter().find(|event|{event._type=="wasm-buy_now"}).unwrap();
            
            let get_nft_ids_and_indexs=|event:&tx_rp_struct::Event|->HashMap<String,usize>{
                let mut nft_ids_and_indexs:HashMap<String,usize>=HashMap::new();
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id"{
                        nft_ids_and_indexs.insert(attr.value.to_owned(), index.to_owned());
                    }
                });
                nft_ids_and_indexs
            };

            let nft_ids_and_indexs=get_nft_ids_and_indexs(wasm_event);
            for (nft_id,nft_id_index) in nft_ids_and_indexs{
                wasm_buy_now_event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.key=="token_id" && attr.value==nft_id{
                        x.push(
                            nft_data_struct::PurchaseCart { 
                                collection:  wasm_event.attributes[nft_id_index-4].value.to_owned(),
                                sender: wasm_event.attributes[nft_id_index-2].value.to_owned(), 
                                recipient: wasm_event.attributes[nft_id_index-1].value.to_owned(), 
                                nft_id: nft_id.to_owned(), 
                                buyer: wasm_buy_now_event.attributes[index+1].value.to_owned(), 
                                seller: wasm_buy_now_event.attributes[index+2].value.to_owned(), 
                                sale_price: wasm_buy_now_event.attributes[index+3].value.to_owned(), 
                                marketplace_fee: wasm_buy_now_event.attributes[index+4].value.to_owned(), 
                                royalties: wasm_buy_now_event.attributes[index+5].value.to_owned(), 
                                transaction_sender:transaction_sender.to_owned(),
                                fee: transaction_fee.to_owned(), 
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                    }
                })
            };

            TransactionEvent::NftPurchaseCart(x)
        
        }else if self.is_token_height_swap() {

            let mut x:Vec<token_data_struct::TokenSwap>=vec![];

            let get_swap_indexs=|event:&tx_rp_struct::Event|->Vec<usize>{
                let mut indexs:Vec<usize>=vec![];
                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.value=="swap"{
                        indexs.push(index)
                    }
                });
                indexs
            };

            let wasm_event=self.events.iter().find(|event| event._type=="wasm").unwrap();
            let swap_indexs=get_swap_indexs(wasm_event);

            for swap_index in swap_indexs{
                x.push(
                    token_data_struct::TokenSwap{
                        source_token:wasm_event.attributes[swap_index+3].value.to_owned(),
                        target_token:wasm_event.attributes[swap_index+4].value.to_owned(),
                        source_amount:wasm_event.attributes[swap_index+5].value.to_owned(),
                        target_amount:wasm_event.attributes[swap_index+6].value.to_owned(),
                        transaction_sender:transaction_sender.to_owned(),
                        fee: transaction_fee.to_owned(), 
                        ts:ts.to_owned(),
                        tx:tx.to_owned(),
                    }
                )
            }

            TransactionEvent::TokenHeihtSwap(x)
        
        }else if self.is_token_normal_swap() {

            let mut x:Vec<token_data_struct::TokenSwap>=vec![];

            let get_swap_indexs=|event:&tx_rp_struct::Event|->Vec<usize>{
                let mut indexs:Vec<usize>=vec![];

                event.attributes.iter().enumerate().for_each(|(index,attr)|{
                    if attr.value=="swap"{
                        indexs.push(index)
                    }
                });
                indexs
            };

            let wasm_event=self.events.iter().find(|event| event._type=="wasm").unwrap();
            let swap_indexs=get_swap_indexs(wasm_event);

            for swap_index in swap_indexs{
                x.push(
                    token_data_struct::TokenSwap{
                        source_token:wasm_event.attributes[swap_index+3].value.to_owned(),
                        target_token:wasm_event.attributes[swap_index+4].value.to_owned(),
                        source_amount:wasm_event.attributes[swap_index+5].value.to_owned(),
                        target_amount:wasm_event.attributes[swap_index+6].value.to_owned(),
                        transaction_sender:transaction_sender.to_owned(),
                        fee: transaction_fee.to_owned(), 
                        ts:ts.to_owned(),
                        tx:tx.to_owned(),
                    }
                )
            }
            TransactionEvent::TokenNormalSwap(x)
        
        }else if self.is_evm_token_transfer_by_wei() {

            let mut x:Vec<token_data_struct::TokenTransfer>=Vec::new();

            let coin_received_event=self.events.iter().find(|event| event._type=="coin_received").unwrap();
            let coin_spent_event=self.events.iter().find(|event| event._type=="coin_spent").unwrap();
            let wei_received_event=self.events.iter().find(|event| event._type=="wei_received").unwrap();

            let (_,transaction_sender_attr)=wei_received_event.attributes.iter().enumerate().find(|(index,attr)|{
                index==&0 && attr.key=="receiver"
            }).unwrap();
            let transaction_sender=transaction_sender_attr.value.to_owned();

            for index in 0..coin_received_event.attributes.len().min(coin_spent_event.attributes.len()){
               
               // 边界判断
               if index*2+1>coin_spent_event.attributes.len(){
                    break;
               };
               
                if coin_spent_event.attributes[index*2].key=="spender" && coin_received_event.attributes[index*2].key=="receiver" &&coin_received_event.attributes[index*2+1].value==coin_spent_event.attributes[index*2+1].value{
                    x.push(
                        token_data_struct::TokenTransfer{
                            sender:coin_spent_event.attributes[index*2].value.to_owned(),
                            receiver:coin_received_event.attributes[index*2].value.to_owned(),
                            amount:coin_received_event.attributes[index*2+1].value.to_owned(),
                            transaction_sender:Some(transaction_sender.to_owned()),
                            fee: transaction_fee.to_owned(), 
                            ts:ts.to_owned(),
                            tx:tx.to_owned(),
                        }
                    )
                }
            }

            
           

            TransactionEvent::TokenTransferByWei(x)

        }else if self.is_token_transfer_by_bank() {

            let mut x:Vec<token_data_struct::TokenTransfer>=Vec::new();

            let transaction_sender=self.events.iter().find(|event| event._type=="message").unwrap().attributes.iter().find(|attr|attr.key=="sender").unwrap().value.to_string();
            
            self.events.iter().for_each(|event|{
                if event._type=="transfer"{
                    for index in 0..event.attributes.len(){
                        if index*3+1>event.attributes.len(){
                            break;
                        };
                        if event.attributes[index*3].key=="recipient" && event.attributes[index*3+1].key=="sender" && event.attributes[index*3+2].key=="amount"{
                            x.push(
                                token_data_struct::TokenTransfer{
                                    sender:event.attributes[index*3].value.to_owned(),
                                    receiver:event.attributes[index*3+1].value.to_owned(),
                                    amount:event.attributes[index*3+2].value.to_owned(),
                                    transaction_sender:Some(transaction_sender.to_owned()),
                                    fee: transaction_fee.to_owned(), 
                                    ts:ts.to_owned(),
                                    tx:tx.to_owned(),
                                }
                            )
                        }
                    }
                }
            });
            
            TransactionEvent::TokenTransferByBank(x)
        
        }else if self.is_token_transfer_by_contract() {
            
            let mut x:Vec<token_data_struct::ContractTokenTransfer>=Vec::new();

            let wasm_event=self.events.iter().find(|event| event._type=="wasm").unwrap();
            
            for index in 0..wasm_event.attributes.len(){      
                if index*5+1>wasm_event.attributes.len(){
                    break;
                };
                if  wasm_event.attributes[index*5].key=="_contract_address"&&
                    wasm_event.attributes[index*5+1].key=="action" && wasm_event.attributes[index*5+1].value=="transfer" &&
                    wasm_event.attributes[index*5+4].key=="amount"{
                        x.push(
                            token_data_struct::ContractTokenTransfer{
                                contract_address:wasm_event.attributes[index*5].value.to_owned(),
                                sender:wasm_event.attributes[index+2].value.to_owned(),
                                receiver:wasm_event.attributes[index+3].value.to_owned(),
                                amount:wasm_event.attributes[index+4].value.to_owned(),
                                transaction_sender:transaction_sender.to_owned(),
                                fee: transaction_fee.to_owned(), 
                                ts:ts.to_owned(),
                                tx:tx.to_owned(),
                            }
                        )
                }
            }
    
            TransactionEvent::TokenTransferByContract(x)
        }else if self.is_delegate() {
            
            let mut x:Vec<stake_data_sturct::Stake>=Vec::new();
            let message_event=self.events.iter().find(|event| event._type=="message").unwrap();
            let transaction_sender= message_event.attributes.iter().find(|attr| attr.key=="sender").map(|attr| attr.value.clone()).unwrap();
            
            let delegate_event=self.events.iter().find(|event| event._type=="delegate").unwrap();

            delegate_event.attributes.iter().enumerate().for_each(|(index,attr)|{
                if attr.key=="validator"{
                    x.push(
                        stake_data_sturct::Stake{
                            validator_address:delegate_event.attributes[index].value.to_owned(),
                            delegator_address:transaction_sender.clone(),
                            amount:delegate_event.attributes[index+1].value.to_owned(),
                            _type:stake_data_sturct::StakeType::Delegate,
                            transaction_sender:Some(transaction_sender.to_owned()),
                            fee: transaction_fee.to_owned(), 
                            ts:ts.to_owned(),
                            tx:tx.to_owned(),
                        }
                    )
                }
            });

            TransactionEvent::Delegate(x)
        }else if self.is_undelegate() {

            let mut x:Vec<stake_data_sturct::Stake>=Vec::new();
            let message_attrs=self.events.iter().find(|event| event._type=="message").unwrap().attributes.to_owned();
            let transaction_sender=message_attrs.last().unwrap().value.to_owned();

            self.events.iter().for_each(|event|{
                if event._type=="unbond"{
                    event.attributes.iter().enumerate().for_each(|(index,attr)|{
                        if attr.key=="validator"{
                            x.push(
                                stake_data_sturct::Stake{
                                    validator_address:event.attributes[index].value.to_owned(),
                                    delegator_address:transaction_sender.to_owned(),
                                    amount:event.attributes[index+1].value.to_owned(),
                                    _type:stake_data_sturct::StakeType::Undelegate,
                                    transaction_sender:Some(transaction_sender.to_owned()),
                                    fee: transaction_fee.to_owned(), 
                                    ts:ts.to_owned(),
                                    tx:tx.to_owned(),
                                }
                            )
                        }
                    })
                }
            });
            TransactionEvent::Undelegate(x)
        
        }else {
            TransactionEvent::Unkonw
        }
    }


    fn is_nft_mint(&self)->bool{
        self.events.iter().any(|event|{
            event._type=="wasm" && event.attributes.iter().any(|attr| attr.value=="mint_nft" || attr.value=="mint")
        })
    }
    
    fn is_nft_batch_bids(&self)->bool {
        if self.events.iter().any(|event|{
            event._type=="wasm"  && event.attributes.iter().any(|attr| attr.value=="batch_bids") &&event.attributes.iter().any(|attr| attr.value=="transfer_nft") 
        }) && self.events.iter().any(|event| event._type=="wasm-buy_now"){
            return true;
        }else {
            return false;
        }
    }
    
    fn is_nft_fixed_sell(&self)->bool {
        self.events.iter().any(|event| event._type=="wasm" && event.attributes.iter().any(|attr| attr.value =="fixed_sell"))
    }
    
    fn is_nft_only_transfer(&self)->bool {
        self.events.iter().any(|event| event._type=="wasm" && event.attributes.iter().any(|attr| attr.value =="transfer_nft") && event.attributes[1].value=="transfer_nft") && self.events.last().unwrap()._type=="wasm"
    }
    
    fn is_nft_create_auction(&self)->bool {
        self.events.iter().any(|event|event._type=="wasm") && self.events.iter().any(|event| event._type=="wasm-create_auction")
    }
    
    fn is_nft_cancel_auction(&self)->bool {
        self.events.iter().any(|event|event._type=="wasm") && self.events.iter().any(|event| event._type=="wasm-cancel_auction")
    }
    
    fn is_nft_purchase_nft(&self)->bool {
        self.events.iter().any(|event|{event._type=="wasm" && event.attributes.iter().any(|attr| attr.value =="purchase_cart") && event.attributes.iter().any(|attr| attr.value=="transfer_nft")})
    }
    
    fn is_nft_only_create_auction(&self)->bool {
        self.events.iter().any(|event| event._type =="wasm-create_auction")
    }
    
    fn is_nft_accept_bid(&self)->bool {
        self.events.iter().any(|event| event._type=="wasm") && self.events.iter().any(|event| event._type=="wasm-accept_bid")
    }

    fn is_token_height_swap(&self) -> bool {
        self.events.iter().any(|event|{event._type=="wasm" && event.attributes.iter().any(|attr| 
            attr.value=="execute_swap_and_action" ||
            attr.value=="dispatch_user_swap" ||
            attr.value=="dispatch_post_swap_action" ||
            attr.value=="execute_user_swap"
        )})
    }

    fn is_token_normal_swap(&self)->bool{
        self.events.iter().any(|event|{event._type=="wasm" && event.attributes.iter().any(|attr| attr.value=="swap")})
    }
    
    fn is_evm_token_transfer_by_wei(&self)->bool {
        self.events.iter().any(|event| event._type=="coin_received") && 
        self.events.iter().any(|event| event._type=="coin_spent") &&
        self.events.iter().any(|event| event._type=="wei_received") &&
        self.events.iter().any(|event|{event._type=="message" && event.attributes.iter().any(|attr|attr.value=="/seiprotocol.seichain.evm.MsgEVMTransaction")})
    }
    
    fn is_token_transfer_by_bank(&self)->bool{
        self.events.iter().any(|event|{
            event._type=="message" && 
            event.attributes.iter().any(|attr| attr.key=="action" && attr.value=="/cosmos.bank.v1beta1.MsgSend") &&
            event.attributes.iter().any(|attr| attr.key=="module" && attr.value=="bank")
        }&& self.events.iter().any(|event| event._type=="transfer"))
    }

    fn is_token_transfer_by_contract(&self)->bool {
        self.events.iter().any(|event| {
            event._type=="wasm" && 
            event.attributes.iter().enumerate().any(|(index,attr)|{
            attr.key=="action" && 
            attr.value=="transfer" &&
            event.attributes[index+3].key=="amount"
        })})
    }
    
    fn is_delegate(&self)->bool {
        self.events.iter().any(|event|{event._type=="message" 
        && event.attributes.iter().any(|attr| attr.key=="action" &&
        attr.value=="/cosmos.staking.v1beta1.MsgDelegate"
        )})
    }
    
    fn is_undelegate(&self)->bool {
        self.events.iter().any(|event|{event._type=="message" 
        && event.attributes.iter().any(|attr| attr.key=="action" &&
        attr.value=="/cosmos.staking.v1beta1.MsgUndelegate"
        )})
    }

}

