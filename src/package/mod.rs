
pub struct AtcScopePackage {
    pub facilities: Vec<AtcFacility>,
}

pub struct AtcFacility {
    pub displays: Vec<AtcDisplay>,
    pub child_facilities: Vec<AtcFacility>
}

pub struct AtcDisplay {

}