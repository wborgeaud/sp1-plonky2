//! A simple script to generate and verify the proof of a given program.

use sp1_sdk::{ProverClient, SP1Stdin, utils};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig, KeccakGoldilocksConfig};
use plonky2::plonk::circuit_data::CircuitData;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::util::serialization::DefaultGateSerializer;

const D: usize = 2;
type C = KeccakGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

fn fibonacci_proof(num_query_rounds: usize, num_challenges: usize) -> (CircuitData<F,C,D>, ProofWithPublicInputs<F,C,D>) {
    let mut config = CircuitConfig::standard_recursion_config();
    config.fri_config.num_query_rounds = num_query_rounds;
    config.security_bits = 1;
    config.num_challenges = num_challenges;
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // The arithmetic circuit.
    let initial_a = builder.add_virtual_target();
    let initial_b = builder.add_virtual_target();
    let mut prev_target = initial_a;
    let mut cur_target = initial_b;
    for _ in 0..99 {
        let temp = builder.add(prev_target, cur_target);
        prev_target = cur_target;
        cur_target = temp;
    }

    // Public inputs are the two initial values (provided below) and the result (which is generated).
    builder.register_public_input(initial_a);
    builder.register_public_input(initial_b);
    builder.register_public_input(cur_target);

    // Provide initial values.
    let mut pw = PartialWitness::new();
    pw.set_target(initial_a, F::ZERO);
    pw.set_target(initial_b, F::ONE);

    let data = builder.build::<C>();
    let proof = data.prove(pw).unwrap();

    (data, proof)
}

fn sp1_prove(circuit: CircuitData<F,C,D>, proof: ProofWithPublicInputs<F, C, D>) {
    let mut stdin = SP1Stdin::new();
    let proof_bytes = proof.to_bytes();
    stdin.write(&proof_bytes);
    // stdin.write(&proof);
    let gate_serializer = DefaultGateSerializer;
    let circuit_bytes = circuit.verifier_data().to_bytes(&gate_serializer).unwrap();
    stdin.write(&circuit_bytes);
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    let proof = client.prove(&pk, stdin).expect("proving failed");

    // Verify proof.
    client.verify(&proof, &vk).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("successfully generated and verified proof for the program!");
}

fn sp1_prove_groth16(circuit: CircuitData<F,C,D>, proof: ProofWithPublicInputs<F, C, D>) {
    let mut stdin = SP1Stdin::new();
    let proof_bytes = proof.to_bytes();
    stdin.write(&proof_bytes);
    let gate_serializer = DefaultGateSerializer;
    let circuit_bytes = circuit.verifier_data().to_bytes(&gate_serializer).unwrap();
    dbg!(circuit_bytes.len());
    stdin.write(&circuit_bytes);
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    let proof = client.prove_groth16(&pk, stdin).unwrap();

    // Verify proof.
    client
        .verify_groth16(&proof, &vk)
        .expect("verification failed");

    // Save the proof.
    proof
        .save("proof-with-pis.json")
        .expect("saving proof failed");

    println!("successfully generated and verified proof for the program!")
}


fn main() {
    utils::setup_logger();
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3, "Pass two arguments: num_query_rounds: usize, num_challenges: usize");
    let num_query_rounds: usize = args[1].parse().unwrap();
    let num_challenges: usize = args[2].parse().unwrap();
    let (circuit, proof) = fibonacci_proof(num_query_rounds, num_challenges);
    assert!(circuit.verify(proof.clone()).is_ok());
    // sp1_prove_groth16(circuit, proof)
    sp1_prove(circuit, proof)
}
