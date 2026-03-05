/// SHACL test 45: TwistedType with non-trivial LiftObstruction — Amendment 30 (MN_7).
pub const TEST45_MONODROMY_TWISTED: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

type:ex_twisted_45 a owl:NamedIndividual, type:TwistedType .

observable:ex_holonomy_45 a owl:NamedIndividual, observable:HolonomyGroup ;
    observable:holonomyGroupOrder  "4"^^xsd:positiveInteger .

observable:ex_monodromy_45 a owl:NamedIndividual, observable:Monodromy ;
    observable:isTrivialMonodromy  "false"^^xsd:boolean .

type:ex_obstruction_45 a owl:NamedIndividual, type:LiftObstruction ;
    type:obstructionTrivial  "false"^^xsd:boolean .

observable:ex_obstruction_class_45 a owl:NamedIndividual, observable:LiftObstructionClass .

observable:ex_dihedral_45 a owl:NamedIndividual, observable:DihedralElement ;
    observable:isIdentityElement  "false"^^xsd:boolean ;
    observable:elementOrder       "2"^^xsd:positiveInteger .
"#;
