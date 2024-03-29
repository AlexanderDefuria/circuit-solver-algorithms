# Circuit Solver Algorithms

## SEG 4105
*Note that this repository is concurrently being evaluated as a part of SEG 4105 Project Mangement. \
As such it will house all project management artifacts throughout the term. \ 
All project management is done through GitHub issues.*

Alexander De Furia - Student Number: 300190815 

## Description
This is the repository for the core circuit solver algorithm and supporting calculations. 
The main concept is to create a circuit solver that can be used in a variety of applications for elementary circuit analysis.
This API Conceptualizes a literal circuit as a container of elements and tools.
Elements being the physical components of the circuit and tools being the conceptual components of the circuit.

A user can provide a container of elements and tools and the API will solve the circuit and return the results.


## Container Rework
This project has aspects that are not ideal and should be reworked. There are pitfalls in the
integration between the `solver` and `container` modules. The `container` module should be
reworked to be more of a data structure and less of a solver. The `solver` module should be
reworked to be more of a solver and less of a data structure. There is no mutability of the container
contents from the solver module. This is a problem because the solver module is the only module that
is intended to be called by the user. It should handle all of the solving and by times this involves
updating the values of the container contents (elements and tools). This is not possible with the current
implementation.



## WASM API
The bulk of the API is defined within [interfaces.rs](./src/inerfaces.rs).
#### Load Container
- `load_wasm_container(container_object) -> Result<String, StatusError> `
- This can be used as a test to see if the container is being loaded in properly.
- It should follow the predefined conventions for the container shape and structure.
- If there are errors it will return said errors.

#### Solve
- Note this is untested and may not work.
- `solve_wasm_container(matrix_bool, nodal_bool, container_object) -> Result<String, StatusError>`
- This will solve the container and return the results as a string.
- This will be the main interface for the WASM API.
- Subject to rapid change, should not be relied on as of now

#### Matrix 
- `return_solved_matrix_example() -> String`
- This returns a constant string of a solved step example.
- Should return latex(?) string of the matrix. (To Confirm).
- This is a constant string for testing purposes.
- This should be removed in the future.

#### Nodal
- `return_solved_nodal_example() -> String`
- This returns a constant string of a solved step example.
- Should return latex(?) string of the nodal. (To Confirm).
- This is the same as the matrix example but for nodal.
- This is a constant string for testing purposes.
- This should be removed in the future.

## WASM Data Structures
There is slight variation between the front and backend due to the restrictions of WASM. Most notably 
for our purposes one cannot pass a struct from the front end to the backend. This is due to the fact that
vector types are not supported in WASM. This means that the container object must be passed as a JSON string

## General Overview
### Containers
The container.rs file contains the implementation of the Container struct, which represents
a collection of Elements and Tools used to solve a circuit. The Container struct has a number
of methods for adding and removing elements and tools, as well as for solving the circuit. 
The Container struct also has a ground field, which represents the index of the ground element
in the elements vector. The Container struct is used extensively throughout the rest of the
crate to represent the circuit being solved.
### Elements
An Element represents a physical component of the circuit, such as a resistor or capacitor.
It has a set of properties, such as its resistance or capacitance, and can be connected to 
other elements to form a circuit.
### Tools
A Tool represents a virtual component of the circuit, such as a node or mesh. It is used as 
conceptual tool to help solve the circuit, and is not a physical component of the circuit.
It has a set of properties, such as its voltage or current, and contains elements and potentially other tools.
### Validation 
There are several validation rules for circuits in this crate of which multiple can be noted for a particular circuit.
Here are a few examples:
- All elements must have unique names.
- All nodes must be connected to at least one other node.
- The circuit must have exactly one ground element.
- The circuit must not contain any loops.
- The circuit must not contain any short circuits.
- The circuit must not contain any open circuits.

The full list of validation rules can be found in the validation.rs file as well as within respective areas of the code.
There are inevitably going to be more rules added as the project progresses. 



### Testing Fixtures
create_basic_container()<br>
![img.png](.github%2Fcreate_basic_container.png)

create_basic_supernode_container()<br>
![img.png](.github%2Fcreate_basic_supernode_container.png)

create_basic_supermesh_container()<br>
![img.png](.github%2Fcreate_basic_supermesh_container.jpg)

create_mna_container()<br>
![img.png](.github%2Fcreate_mna_container.png)

<style type="text/css">
    img {
        width: 400px;
    }
</style>
