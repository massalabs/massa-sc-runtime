# Hello world 🌍

Hello world project for Massa's smart contracts development. This preset environment can be used to test and debug locally your smart contract before sending it to the [Massa network](https://github.com/massalabs/massa).

## Quick setup 🧰

You need `node`, `npm` and `npx` to initialize the project!

```shell
npx massa-sc-create hello-world
```

> N.B. if you didn't initialize this project with `npx`, you can clone and customize manually this repository with the command below. You'll be able to change the version of `massa-sc-std` if it's not already at the latest version.
>
> ```shell
> git clone https://github.com/massalabs/massa-sc-template
> ```

Once this repository is cloned, run the following command in the freshly created directory:

```shell
npm install
```

## Usage ✍️

You can run scripts described in the package.json with `yarn run {script_name}`, if you're not confident with the package.json, take a minute to look at the [yarn documentation](https://classic.yarnpkg.com/lang/en/docs/cli/run/).

I'll describe the different embedded scripts in this section and redirect to several documentations if required.

-   `build`: generate a `.wasm` file in a `build/` directory for `src/smart-contract.ts` and `src/main.ts`. This is the hello world smart contract example. If you look at this command line, you can see that we use the `asc` binary, this is the AssemblyScript compiler. Here is [the documentation](https://www.assemblyscript.org/introduction.html) if you want to know more. The `build/smart-contract.wasm` is a simple smart-contract that can be sent to the blockchain and will be executed once and then drop. The `build/main.wasm` will store this hello world on the blockchain and so you will be able to call it whenever you want.
-   `clean`: remove build artifacts and temporary ledger file.
-   `test`: _(not yet implemented)_ execute the smart contract on a mock with the `massa-sc-test` binary. Look at the mock repository [here](https://github.com/massalabs/massa-sc-tester) for more information about mocking the network.
-   `publish`: _(not yet implemented)_ publish `bin/main.wasm` smart contract to a given Massa network node.

## Mocked ledger 👛

To test your smart contracts locally, [download](https://github.com/massalabs/massa-sc-tester/releases) the `massa-sc-tester` release that corresponds to your environment. Unzip the downloaded file and put the executable in `./bin` directory. Make sure that the right access permissions are set with `chmod`.

Once you ran the `exec` script, you should see a new file in the directory named `ledger.json`. This file represents the local state of the ledger for your test. You can modify it manually (obviously, carefully) to look at the state of the ledger after each execution.

> For now, the execution on the mocked ledger doesn't behave exactly like on the real massa network, in particular if the execution of a smart-contract fails. The ledger file is modified each time you run a code that writes on the ledger file and there is no backup management. It means that even if your code failed after writing in the ledger, the modifications are saved while the execution would be reverted on the real massa network.

## Report an issue 🚩

If you get an issue with `massa-sc-test` we would appreciate a report on the [dedicated repository](https://github.com/massalabs/massa-sc-tester/issues/new/choose). Use an appropriate language and explain step by step your problem with examples and screenshots.
