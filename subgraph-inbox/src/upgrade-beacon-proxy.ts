import { BigInt, log } from "@graphprotocol/graph-ts"
import {
  UpgradeBeaconProxy,
  Initialized,
  OwnershipTransferred,
  Process,
  ValidatorManagerSet
} from "../generated/UpgradeBeaconProxy/UpgradeBeaconProxy"
import { RelayerExpensesDaily } from "../generated/schema"

export function handleProcess(event: Process): void {

  let timestamp = event.block.timestamp.toI32()
  let dayID = timestamp / 86400
  let dayIDFormatted = BigInt.fromI32(dayID).toString()

  //user and date
  const day = event.block.timestamp / BigInt.fromI32(86400)
  const date = day * BigInt.fromI32(86400)
log.info("2 {}", [date.toString()])
  let dayData = RelayerExpensesDaily.load(dayIDFormatted)
  if (dayData == null) {
    dayData = new RelayerExpensesDaily(dayIDFormatted)
    if (event.transaction != null && event.transaction.gasUsed){
      dayData.amount = event.transaction.gasUsed
    } else {
      log.info("Was null", [])
      dayData.amount = BigInt.fromI32(0)
    }
    dayData.transactionAmount = BigInt.fromI32(1)
    log.info("3", [])
    dayData.date = date
    log.info("4", [])
  } else {
    log.info("ELSE", [])
    if (event.transaction != null && event.transaction.gasUsed){
      dayData.amount = dayData.amount + event.transaction.gasUsed
      log.info("Has receipt .. gas used {}", [event.transaction.gasUsed.toString()])
    } else {
      log.info("NO Receipt", [])
    }
    dayData.transactionAmount = dayData.transactionAmount + BigInt.fromI32(1)
  }
log.info("5. id {}. amount {}. transactionAmount {}. date {}", [dayData.id, dayData.amount.toString(), dayData.transactionAmount.toString(), dayData.date.toString()])
  dayData.save()
log.info("6", [])
}


