use ethcontract_generate::loaders::TruffleLoader;
use ethcontract_generate::ContractBuilder;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    {
        let dest = std::path::Path::new(&out_dir).join("SugarFungeAsset.rs");

        let artifact = TruffleLoader::new()
            .load_from_file("./contracts/SugarFunge/SugarFungeAsset.json")
            .unwrap();

        for contract in artifact.iter() {
            ContractBuilder::new()
                .generate(contract)
                .unwrap()
                .write_to_file(&dest)
                .unwrap();
        }
    }

    {
        let dest = std::path::Path::new(&out_dir).join("Wrapped1155Factory.rs");

        let artifact = TruffleLoader::new()
            .load_from_file("./contracts/ErcWrapper/Wrapped1155Factory.json")
            .unwrap();

        for contract in artifact.iter() {
            ContractBuilder::new()
                .generate(contract)
                .unwrap()
                .write_to_file(&dest)
                .unwrap();
        }
    }
}
