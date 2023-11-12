

enum ElectionState {

}

struct Election {
    voters: Vec<String>,
}

impl Election {
    fn new(votes: Vec<String>) -> Self {
        return Election { voters: votes };
    }

    fn leader_election(&self) {
        
    }

    fn find_leader_info() {}

    fn election() {}
}
