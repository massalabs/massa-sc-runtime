/* Some users could forget to export a main function and the following code
   could be executed witthout being initialized.
*/

import {print, call } from "massa-sc-std";

let addr = "jCcqGSAVh9BR5icEk8icdvEqeNZzvsPK4xZK9Fm5PaWFab48X";
call(addr, "vote", "", 2000);
print("DB = " + call(addr, "get_db", "", 0)); 