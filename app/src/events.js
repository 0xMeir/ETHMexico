import OutboxABI from "./outbox.json"
import InboxABI from "./inbox.json"
import { ethers } from "ethers";

export const getAllOutboxEarned = async () =>  {

  let days = {}

  let iface = new ethers.utils.Interface(OutboxABI)
  const provider = new ethers.getDefaultProvider(10);
  const outbox = "0xc5D6aCaafBCcEC6D7fD7d92F4509befce641c563"
  const contract = new ethers.Contract(outbox, OutboxABI, provider);
  let filter = contract.filters.GasPayment();
  let events = await contract.queryFilter(filter, 14609398);

  for (var i=0; i< events.length; i++){
    const timestamp = (await provider.getBlock(events[i].blockNumber)).timestamp;
    var dt = new Date(timestamp * 1000)
    var dtKey = dt.getFullYear() + "/" + (dt.getMonth() + 1) + "/" + dt.getDate();
    // console.log("Timestamp", timestamp, dt, dtKey)

    let interf = iface.parseLog(events[i]);
    // console.log("amount ", interf.args.amount.toNumber())
    if (days[dtKey]){
      days[dtKey] = days[dtKey] + interf.args.amount.toNumber()
    } else {
      days[dtKey] = interf.args.amount.toNumber()
    }
  }

  console.log("days/earned (value in wei)", days)

  return days;

}


export const getAllInboxFees = async () =>  {

  let days = {}

  const provider = new ethers.getDefaultProvider();
  const inbox = "0xf7af65596a16740b16cf755f3a43206c96285da0"
  const contract = new ethers.Contract(inbox, InboxABI, provider);
  let filter = contract.filters.Process();
  let events = await contract.queryFilter(filter, 14970241);
  console.log(events)
  for (var i=0; i< events.length; i++){
    const timestamp = (await provider.getBlock(events[i].blockNumber)).timestamp;
    var dt = new Date(timestamp * 1000)
    var dtKey = dt.getFullYear() + "/" + (dt.getMonth() + 1) + "/" + dt.getDate();


    let tx = await events[i].getTransactionReceipt();
    console.log("tx",tx);
    let spent = tx.effectiveGasPrice.mul(tx.cumulativeGasUsed).div(ethers.BigNumber.from(10).pow(ethers.BigNumber.from(9))).toNumber()
    console.log("spent in wei", spent)

    if (days[dtKey]){
      days[dtKey] = days[dtKey] + spent
    } else {
      days[dtKey] = spent
    }
  }

  console.log("Days/spent (value in eth)", days)

  return days;
}