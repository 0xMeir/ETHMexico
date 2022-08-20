import { BigInt } from "@graphprotocol/graph-ts"
import {
  InterchainGasPaymaster,
  GasPayment,
  Initialized,
  OwnershipTransferred
} from "../generated/InterchainGasPaymaster/InterchainGasPaymaster"
import { RelayerEarningsDaily } from "../generated/schema"

export function handleGasPayment(event: GasPayment): void {
  // Entities can be loaded from the store using a string ID; this ID
  // needs to be unique across all entities of the same type

  let timestamp = event.block.timestamp.toI32()
  let dayID = timestamp / 86400
  let dayIDFormatted = BigInt.fromI32(dayID).toString()

  //user and date
  const day = event.block.timestamp.toI32() / 86400
  const date = day * 86400


  let dayData = RelayerEarningsDaily.load(dayIDFormatted)
  if (dayData == null) {
    dayData = new RelayerEarningsDaily(dayIDFormatted)
    dayData.amount = event.params.amount
    dayData.transactionAmount = BigInt.fromI32(1)
  } else {
    dayData.amount = dayData.amount + event.params.amount
    dayData.transactionAmount = dayData.transactionAmount + BigInt.fromI32(1)
  }

  if (!dayData.date) {
    dayData.date = date
  }

  dayData.save()

}
