use std::{fs::File, io::BufWriter, path::Path};

use directories::UserDirs;

use crate::{loaders::euroscope::loader::EuroScopeLoader, package::AtcScopePackage};


#[test]
fn test_convert_es_path_1(){
    let prf_path = "/a/b/c.prf";
    let es_path = "\\d\\f.txt";

    let computed = EuroScopeLoader::try_convert_es_path(prf_path, es_path).unwrap();
    let expected = Path::new("/a/b/d/f.txt");
    assert_eq!(computed.as_os_str(), expected.as_os_str());
}

#[test]
fn test_convert_es_path_2(){
    let prf_path = "/a/b/c.prf";
    let es_path = "BLAH_FIR\\d\\f.txt";

    let computed = EuroScopeLoader::try_convert_es_path(prf_path, es_path).unwrap();
    let expected = UserDirs::new().unwrap().document_dir().unwrap().join("EuroScope").join("BLAH_FIR/d/f.txt");
    assert_eq!(computed.as_os_str(), expected.as_os_str());
}

#[test]
#[ignore]
fn test_load_es_1(){
    let prf_path = "/Users/pshivaraman/Documents/EuroScope/UK/Belfast/Belfast Combined.prf";
    let mut es = EuroScopeLoader::try_new_from_prf(prf_path).unwrap();
    let result = es.try_read().unwrap();

    let package = AtcScopePackage::try_from(result).unwrap();

    serde_json::to_writer_pretty(BufWriter::new(File::create("test.json").unwrap()), &package);

    let a = package.maps.get("/Users/pshivaraman/Documents/EuroScope/UK/Belfast/Sector/Belfast.sct_regions_Belfast City");
    //println!("{:#?}", es);
}