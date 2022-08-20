import { newMockEvent } from "matchstick-as"
import { ethereum, Address, Bytes } from "@graphprotocol/graph-ts"
import {
  Initialized,
  OwnershipTransferred,
  Process,
  ValidatorManagerSet
} from "../generated/UpgradeBeaconProxy/UpgradeBeaconProxy"

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

export function createProcessEvent(messageHash: Bytes): Process {
  let processEvent = changetype<Process>(newMockEvent())

  processEvent.parameters = new Array()

  processEvent.parameters.push(
    new ethereum.EventParam(
      "messageHash",
      ethereum.Value.fromFixedBytes(messageHash)
    )
  )

  return processEvent
}

export function createValidatorManagerSetEvent(
  validatorManager: Address
): ValidatorManagerSet {
  let validatorManagerSetEvent = changetype<ValidatorManagerSet>(newMockEvent())

  validatorManagerSetEvent.parameters = new Array()

  validatorManagerSetEvent.parameters.push(
    new ethereum.EventParam(
      "validatorManager",
      ethereum.Value.fromAddress(validatorManager)
    )
  )

  return validatorManagerSetEvent
}
