import { newMockEvent } from "matchstick-as"
import { ethereum, Address, BigInt } from "@graphprotocol/graph-ts"
import {
  GasPayment,
  Initialized,
  OwnershipTransferred
} from "../generated/InterchainGasPaymaster/InterchainGasPaymaster"

export function createGasPaymentEvent(
  outbox: Address,
  leafIndex: BigInt,
  amount: BigInt
): GasPayment {
  let gasPaymentEvent = changetype<GasPayment>(newMockEvent())

  gasPaymentEvent.parameters = new Array()

  gasPaymentEvent.parameters.push(
    new ethereum.EventParam("outbox", ethereum.Value.fromAddress(outbox))
  )
  gasPaymentEvent.parameters.push(
    new ethereum.EventParam(
      "leafIndex",
      ethereum.Value.fromUnsignedBigInt(leafIndex)
    )
  )
  gasPaymentEvent.parameters.push(
    new ethereum.EventParam("amount", ethereum.Value.fromUnsignedBigInt(amount))
  )

  return gasPaymentEvent
}

export function createInitializedEvent(version: i32): Initialized {
  let initializedEvent = changetype<Initialized>(newMockEvent())

  initializedEvent.parameters = new Array()

  initializedEvent.parameters.push(
    new ethereum.EventParam(
      "version",
      ethereum.Value.fromUnsignedBigInt(BigInt.fromI32(version))
    )
  )

  return initializedEvent
}

export function createOwnershipTransferredEvent(
  previousOwner: Address,
  newOwner: Address
): OwnershipTransferred {
  let ownershipTransferredEvent = changetype<OwnershipTransferred>(
    newMockEvent()
  )

  ownershipTransferredEvent.parameters = new Array()

  ownershipTransferredEvent.parameters.push(
    new ethereum.EventParam(
      "previousOwner",
      ethereum.Value.fromAddress(previousOwner)
    )
  )
  ownershipTransferredEvent.parameters.push(
    new ethereum.EventParam("newOwner", ethereum.Value.fromAddress(newOwner))
  )

  return ownershipTransferredEvent
}
