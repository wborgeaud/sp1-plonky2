#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    use plonky2::plonk::proof::ProofWithPublicInputs;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::plonk::config::PoseidonGoldilocksConfig;
    use plonky2::plonk::circuit_data::VerifierCircuitData;
    use plonky2::util::serialization::gate_serialization::default::DefaultGateSerializer;

    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    let proof = sp1_zkvm::io::read::<ProofWithPublicInputs<F,C,D>>();
    sp1_zkvm::io::commit(&proof.public_inputs);
    let circuit_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    sp1_zkvm::io::commit(&circuit_bytes);
    let gate_serializer = DefaultGateSerializer;
    let circuit = VerifierCircuitData::from_bytes(circuit_bytes, &gate_serializer).unwrap();

    circuit.verify(proof).unwrap();
}
