
use tracing::Instrument;
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationLayer, AllocationRegistry,
    AllocationTracker, Allocator,
};
use std::alloc::System;
use anyhow::Result;
//use std::sync::Arc;
//use serial_test::serial;

use crate::{
    run_function,
    types::{GasCosts, Interface},
};
use crate::tests::{Ledger, TestInterface};

use parking_lot::Mutex;
use std::sync::Arc;

//use rand::Rng;
use serial_test::serial;

// This is where we actually set the global allocator to be the shim allocator implementation from `tracking_allocator`.
// This allocator is purely a facade to the logic provided by the crate, which is controlled by setting a global tracker
// and registering allocation groups.  All of that is covered below.
//
// As well, you can see here that we're wrapping the system allocator.  If you want, you can construct `Allocator` by
// wrapping another allocator that implements `GlobalAlloc`.  Since this is a static, you need a way to construct ther
// allocator to be wrapped in a const fashion, but it _is_ possible.
#[global_allocator]
static GLOBAL: Allocator<System> = Allocator::system();

struct StdoutTracker;

// This is our tracker implementation.  You will always need to create an implementation of `AllocationTracker` in order
// to actually handle allocation events.  The interface is straightforward: you're notified when an allocation occurs,
// and when a deallocation occurs.
impl AllocationTracker for StdoutTracker {
    fn allocated(
        &self,
        addr: usize,
        object_size: usize,
        wrapped_size: usize,
        group_id: AllocationGroupId,
    ) {
        // Allocations have all the pertinent information upfront, which you may or may not want to store for further
        // analysis. Notably, deallocations also know how large they are, and what group ID they came from, so you
        // typically don't have to store much data for correlating deallocations with their original allocation.
        println!(
            "allocation -> addr=0x{:0x} object_size={} wrapped_size={} group_id={:?}",
            addr, object_size, wrapped_size, group_id
        );
    }

    fn deallocated(
        &self,
        addr: usize,
        object_size: usize,
        wrapped_size: usize,
        source_group_id: AllocationGroupId,
        current_group_id: AllocationGroupId,
    ) {
        // When a deallocation occurs, as mentioned above, you have full access to the address, size of the allocation,
        // as well as the group ID the allocation was made under _and_ the active allocation group ID.
        //
        // This can be useful beyond just the obvious "track how many current bytes are allocated by the group", instead
        // going further to see the chain of where allocations end up, and so on.
        println!(
            "deallocation -> addr=0x{:0x} object_size={} wrapped_size={} source_group_id={:?} current_group_id={:?}",
            addr, object_size, wrapped_size, source_group_id, current_group_id
        );
    }
}

async fn test_vector_alloc() {
    AllocationRegistry::enable_tracking();

    let v:Vec<String> = Vec::with_capacity(10);
    println!("{}", v.len());
    drop(v);
    
    AllocationRegistry::disable_tracking();
}

async fn test_run_function_alloc() {

    AllocationRegistry::enable_tracking();

    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/receive_message.wasm"
    ));

    AllocationRegistry::disable_tracking();

    run_function(
        module,
        100,
        "receive",
        b"data",
        &*interface,
        gas_costs.clone(),
    )
    .expect("Failed to run_function receive_message.wasm");
    
    AllocationRegistry::enable_tracking();

    drop(gas_costs);
    drop(interface);
    drop(module);

    AllocationRegistry::disable_tracking();


}

//#[tokio::test]
#[tokio::test(flavor = "multi_thread")]
async fn test_allocation_vec() -> Result<()> {
    
    // Configure tracing with our [`AllocationLayer`] so that enter/exit events are handled correctly.
    let registry = Registry::default().with(AllocationLayer::new());
    tracing::subscriber::set_global_default(registry)
        .expect("failed to install tracing subscriber");

    // Create and set our allocation tracker.  Even with the tracker set, we're still not tracking allocations yet.  We
    // need to enable tracking explicitly.
    let _ = AllocationRegistry::set_global_tracker(StdoutTracker)
        .expect("no other global tracker should be set yet");

    // Register two allocation groups.  Allocation groups are what allocations are associated with.  and if there is no
    // user-register allocation group active during an allocation, the "root" allocation group is used.  This matches
    // the value returned by `AllocationGroupId::ROOT`.
    //
    // This gives us a way to actually have another task or thread processing the allocation events -- which may require
    // allocating storage to do so -- without ending up in a weird re-entrant situation if we just instrumented all
    // allocations throughout the process.
    let task1_token =
        AllocationGroupToken::register().expect("failed to register allocation group");

    // Even with the tracker set, we're still not tracking allocations yet.  We need to enable tracking explicitly.

    let task1_span = info_span!("task1");
    task1_token.attach_to_span(&task1_span);
    let task1 = test_vector_alloc().instrument(task1_span);
    //let task1 = test_run_function_alloc().instrument(task1_span);
    
    // Now let them run and wait for them to complete.

    let handle1 = tokio::spawn(task1);
    
    let _ = handle1.await.expect("task1 panicked unexpectedly");
    
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_allocation_function() -> Result<()> {
    
    // Configure tracing with our [`AllocationLayer`] so that enter/exit events are handled correctly.
    let registry = Registry::default().with(AllocationLayer::new());
    tracing::subscriber::set_global_default(registry)
        .expect("failed to install tracing subscriber");

    // Create and set our allocation tracker.  Even with the tracker set, we're still not tracking allocations yet.  We
    // need to enable tracking explicitly.
    let _ = AllocationRegistry::set_global_tracker(StdoutTracker)
        .expect("no other global tracker should be set yet");

    // Register two allocation groups.  Allocation groups are what allocations are associated with.  and if there is no
    // user-register allocation group active during an allocation, the "root" allocation group is used.  This matches
    // the value returned by `AllocationGroupId::ROOT`.
    //
    // This gives us a way to actually have another task or thread processing the allocation events -- which may require
    // allocating storage to do so -- without ending up in a weird re-entrant situation if we just instrumented all
    // allocations throughout the process.
    let task1_token =
        AllocationGroupToken::register().expect("failed to register allocation group");

    // Even with the tracker set, we're still not tracking allocations yet.  We need to enable tracking explicitly.

    let task1_span = info_span!("task1");
    task1_token.attach_to_span(&task1_span);
    //let task1 = test_run_function_alloc().instrument(task1_span);
    let task1 = test_run_function_alloc();
    
    // Now let them run and wait for them to complete.

    let handle1 = tokio::spawn(task1);
    
    let _ = handle1.await.expect("task1 panicked unexpectedly");
    
    Ok(())
}