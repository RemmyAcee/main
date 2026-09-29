#![cfg(test)]

use super::*;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Bytes, Env};

#[contract]
struct MockLineageVerifier;

#[contractimpl]
impl MockLineageVerifier {
    pub fn verify_proof(_env: Env, _public_inputs: Bytes, _proof: Bytes) {}
}

#[test]
fn registers_lineage_with_bounded_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    let parent = bytes32(&env, 1);

    client.init(&admin);
    client.register_source(&actor, &bytes32(&env, 2), &bytes32(&env, 3), &parent);

    let lineage = client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [parent.clone()]),
        &bytes32(&env, 4),
        &Symbol::new(&env, "crop"),
        &bytes32(&env, 5),
        1,
    );

    assert_eq!(lineage.output_digest, bytes32(&env, 5));
    assert_eq!(lineage.depth, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn rejects_excessive_fanout() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    client.init(&admin);

    let parents = soroban_sdk::Vec::from_array(
        &env,
        [
            bytes32(&env, 1),
            bytes32(&env, 2),
            bytes32(&env, 3),
            bytes32(&env, 4),
            bytes32(&env, 5),
        ],
    );

    client.register_lineage(
        &actor,
        &parents,
        &bytes32(&env, 6),
        &Symbol::new(&env, "compose"),
        &bytes32(&env, 7),
        1,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn rejects_self_referential_lineage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    client.init(&admin);

    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [bytes32(&env, 1)]),
        &bytes32(&env, 2),
        &Symbol::new(&env, "crop"),
        &bytes32(&env, 1),
        1,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn rejects_direct_cycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    client.init(&admin);

    let p1 = bytes32(&env, 1);
    let p2 = bytes32(&env, 2);

    client.register_source(&actor, &bytes32(&env, 5), &bytes32(&env, 6), &p1);

    // Register p2 deriving from p1
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [p1.clone()]),
        &bytes32(&env, 10),
        &Symbol::new(&env, "crop"),
        &p2,
        1,
    );

    // Register p1 deriving back from p2 -> direct cycle between 2 nodes
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [p2.clone()]),
        &bytes32(&env, 11),
        &Symbol::new(&env, "crop"),
        &p1,
        2,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn rejects_transitive_cycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HarpocratesRegistry, ());
    let client = HarpocratesRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let actor = Address::generate(&env);
    client.init(&admin);

    let p1 = bytes32(&env, 1);
    let p2 = bytes32(&env, 2);
    let p3 = bytes32(&env, 3);
    let p4 = bytes32(&env, 4);

    client.register_source(&actor, &bytes32(&env, 5), &bytes32(&env, 6), &p1);

    // p1 -> p2
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [p1.clone()]),
        &bytes32(&env, 10),
        &Symbol::new(&env, "crop"),
        &p2,
        1,
    );

    // p2 -> p3
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [p2.clone()]),
        &bytes32(&env, 11),
        &Symbol::new(&env, "crop"),
        &p3,
        2,
    );

    // p3 -> p4
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [p3.clone()]),
        &bytes32(&env, 12),
        &Symbol::new(&env, "crop"),
        &p4,
        3,
    );

    // transitive cycle: p4 -> p1
    client.register_lineage(
        &actor,
        &soroban_sdk::Vec::from_array(&env, [p4.clone()]),
        &bytes32(&env, 13),
        &Symbol::new(&env, "crop"),
        &p1,
        4, // MAX_LINEAGE_DEPTH is 4, this is valid depth, but triggers the cycle error
    );
}
