/**
 * Hello world
 */

import {
    establishConnection,
    establishPayer,
    checkAccounts,
    testContract,
} from './amoebit_init';

async function main() {
    // Establish connection to the cluster
    await establishConnection();

    // our dev wallet
    await establishPayer();

    // init the index account
    await checkAccounts();

    // run the contract
    await testContract();
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
