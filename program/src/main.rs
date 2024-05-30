#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    use plonky2::plonk::proof::ProofWithPublicInputs;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::plonk::config::PoseidonGoldilocksConfig;
    use plonky2::plonk::config::KeccakGoldilocksConfig;
    use plonky2::plonk::circuit_data::VerifierCircuitData;
    use plonky2::util::serialization::gate_serialization::default::DefaultGateSerializer;

    type F = GoldilocksField;
    type C = KeccakGoldilocksConfig;
    const D: usize = 2;

    let proof_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    let circuit_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    sp1_zkvm::io::commit(&circuit_bytes);
    let gate_serializer = DefaultGateSerializer;
    let circuit = VerifierCircuitData::from_bytes(circuit_bytes, &gate_serializer).unwrap();
    let proof = ProofWithPublicInputs::<F,C,D>::from_bytes(proof_bytes, &circuit.common).unwrap();
    sp1_zkvm::io::commit(&proof.public_inputs);

    circuit.verify(proof).unwrap();
}
