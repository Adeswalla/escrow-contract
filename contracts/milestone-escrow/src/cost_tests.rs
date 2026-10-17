#![cfg(test)]

use crate::{MilestoneEscrow, MilestoneEscrowClient};
use soroban_sdk::{
    testutils::{Address as _, EnvTestConfig, Ledger as _}, token, vec, Address, Env,
};

const MILESTONE_AMOUNT: i128 = 1_000;
const AUTO_RELEASE_SECONDS: u64 = 604_800;
const INITIAL_TIMESTAMP: u64 = 1_700_000_000;

struct Fixture<'a> {
    client_addr: Address,
    freelancer_addr: Address,
    arbiter_addr: Address,
    admin_addr: Address,
    token_contract_id: Address,
    escrow: MilestoneEscrowClient<'a>,
}

#[derive(Clone, Copy)]
struct Cost {
    name: &'static str,
    instructions: u64,
    memory_read_entries: u32,
    memory_read_bytes: u32,
    disk_read_entries: u32,
    disk_read_bytes: u32,
    write_entries: u32,
    write_bytes: u32,
}

fn fixture(env: &Env) -> Fixture<'_> {
    env.ledger().set_timestamp(INITIAL_TIMESTAMP);
    let client_addr = Address::generate(env);
    let freelancer_addr = Address::generate(env);
    let arbiter_addr = Address::generate(env);
    let admin_addr = Address::generate(env);
    let token_contract_id = env
        .register_stellar_asset_contract_v2(admin_addr.clone())
        .address();
    token::StellarAssetClient::new(env, &token_contract_id).mint(&client_addr, &MILESTONE_AMOUNT);
    let contract_id = env.register(MilestoneEscrow, ());
    let escrow = MilestoneEscrowClient::new(env, &contract_id);

    Fixture {
        client_addr,
        freelancer_addr,
        arbiter_addr,
        admin_addr,
        token_contract_id,
        escrow,
    }
}

fn initialize(fixture: &Fixture<'_>, env: &Env) {
    fixture.escrow.initialize(
        &fixture.admin_addr,
        &fixture.client_addr,
        &fixture.freelancer_addr,
        &fixture.arbiter_addr,
        &fixture.token_contract_id,
        &AUTO_RELEASE_SECONDS,
        &vec![env, MILESTONE_AMOUNT],
    );
}

fn fund(fixture: &Fixture<'_>) {
    fixture.escrow.fund(&fixture.client_addr);
}

fn measure(name: &'static str, env: &Env) -> Cost {
    let resources = env.cost_estimate().resources();
    Cost {
        name,
        instructions: resources.instructions,
        memory_read_entries: resources.memory_read_entries,
        memory_read_bytes: resources.memory_read_bytes,
        disk_read_entries: resources.disk_read_entries,
        disk_read_bytes: resources.disk_read_bytes,
        write_entries: resources.write_entries,
        write_bytes: resources.write_bytes,
    }
}

fn new_env() -> Env {
    let env = Env::new_with_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    env.mock_all_auths();
    env
}

#[test]
fn main_entrypoint_cost_baselines() {
    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    let initialize_cost = measure("initialize", &env);

    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    fund(&setup);
    let fund_cost = measure("fund", &env);

    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    fund(&setup);
    setup.escrow.mark_delivered(&setup.freelancer_addr, &0);
    let mark_delivered_cost = measure("mark_delivered", &env);

    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    fund(&setup);
    setup.escrow.mark_delivered(&setup.freelancer_addr, &0);
    setup.escrow.approve_milestone(&setup.client_addr, &0);
    let approve_milestone_cost = measure("approve_milestone", &env);

    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    fund(&setup);
    setup.escrow.mark_delivered(&setup.freelancer_addr, &0);
    env.ledger()
        .set_timestamp(INITIAL_TIMESTAMP + AUTO_RELEASE_SECONDS);
    setup.escrow.claim_auto_release(&setup.freelancer_addr, &0);
    let claim_auto_release_cost = measure("claim_auto_release", &env);

    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    fund(&setup);
    setup.escrow.raise_dispute(&setup.client_addr, &0);
    let raise_dispute_cost = measure("raise_dispute", &env);

    let env = new_env();
    let setup = fixture(&env);
    initialize(&setup, &env);
    fund(&setup);
    setup.escrow.raise_dispute(&setup.client_addr, &0);
    setup
        .escrow
        .resolve_dispute(&setup.arbiter_addr, &0, &true);
    let resolve_dispute_cost = measure("resolve_dispute", &env);

    let costs = [
        initialize_cost,
        fund_cost,
        mark_delivered_cost,
        approve_milestone_cost,
        claim_auto_release_cost,
        raise_dispute_cost,
        resolve_dispute_cost,
    ];

    println!("\nMain entry-point resource costs");
    println!(
        "{:<22} {:>14} {:>11} {:>11} {:>10} {:>10} {:>12} {:>12}",
        "flow",
        "instructions",
        "mem reads",
        "mem bytes",
        "disk reads",
        "disk bytes",
        "write entries",
        "write bytes"
    );
    for cost in costs {
        println!(
            "{:<22} {:>14} {:>11} {:>11} {:>10} {:>10} {:>12} {:>12}",
            cost.name,
            cost.instructions,
            cost.memory_read_entries,
            cost.memory_read_bytes,
            cost.disk_read_entries,
            cost.disk_read_bytes,
            cost.write_entries,
            cost.write_bytes
        );
        assert!(cost.instructions > 0, "{} was not metered", cost.name);
    }
}