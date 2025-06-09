use soroban_sdk::{contractimpl, Address, Env, Symbol, Map, contracttype};

pub struct AtomicSwapContract;

#[contracttype]
pub struct Swap {
    pub party_a: Address,
    pub party_b: Address,
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a: i128,
    pub amount_b: i128,
    pub deposited_a: bool,
    pub deposited_b: bool,
    pub completed: bool,
    pub cancelled: bool,
}

#[contractimpl]
impl AtomicSwapContract {
    // Storage: Map<swap_id, Swap>
    fn swaps<'a>(env: &'a Env) -> Map<'a, Symbol, Swap> {
        env.storage().instance().get_map(Symbol::short("swaps"))
    }

    // Create a new swap offer
    pub fn create_swap(
        env: Env,
        swap_id: Symbol,
        party_b: Address,
        token_a: Address,
        token_b: Address,
        amount_a: i128,
        amount_b: i128,
    ) {
        let party_a = env.invoker();
        let mut swaps = Self::swaps(&env);

        if swaps.has(swap_id.clone()) {
            panic!("Swap with this ID already exists.");
        }

        let swap = Swap {
            party_a: party_a.clone(),
            party_b,
            token_a,
            token_b,
            amount_a,
            amount_b,
            deposited_a: false,
            deposited_b: false,
            completed: false,
            cancelled: false,
        };
        swaps.set(swap_id, swap);
        env.storage().instance().set(Symbol::short("swaps"), &swaps);
    }

    // Deposit tokens for the swap
    pub fn deposit(env: Env, swap_id: Symbol) {
        let caller = env.invoker();
        let mut swaps = Self::swaps(&env);
        let mut swap = swaps.get(swap_id.clone()).expect("Swap not found.");

        if swap.completed || swap.cancelled {
            panic!("Swap already completed or cancelled.");
        }

        if caller == swap.party_a && !swap.deposited_a {
            // Here you would call the token contract to transfer amount_a from party_a to this contract
            swap.deposited_a = true;
        } else if caller == swap.party_b && !swap.deposited_b {
            // Here you would call the token contract to transfer amount_b from party_b to this contract
            swap.deposited_b = true;
        } else {
            panic!("Caller is not a swap party or already deposited.");
        }

        swaps.set(swap_id, swap);
        env.storage().instance().set(Symbol::short("swaps"), &swaps);
    }

    // Complete the swap if both parties have deposited
    pub fn complete_swap(env: Env, swap_id: Symbol) {
        let mut swaps = Self::swaps(&env);
        let mut swap = swaps.get(swap_id.clone()).expect("Swap not found.");

        if swap.completed || swap.cancelled {
            panic!("Swap already completed or cancelled.");
        }
        if !swap.deposited_a || !swap.deposited_b {
            panic!("Both parties must deposit before completing the swap.");
        }

        // Here you would call the token contracts to transfer amount_a to party_b and amount_b to party_a
        swap.completed = true;

        swaps.set(swap_id, swap);
        env.storage().instance().set(Symbol::short("swaps"), &swaps);
    }

    // Cancel the swap and allow parties to reclaim their deposits
    pub fn cancel_swap(env: Env, swap_id: Symbol) {
        let caller = env.invoker();
        let mut swaps = Self::swaps(&env);
        let mut swap = swaps.get(swap_id.clone()).expect("Swap not found.");

        if swap.completed || swap.cancelled {
            panic!("Swap already completed or cancelled.");
        }
        if caller != swap.party_a && caller != swap.party_b {
            panic!("Only swap parties can cancel.");
        }

        // Here you would call the token contracts to refund any deposited tokens
        swap.cancelled = true;

        swaps.set(swap_id, swap);
        env.storage().instance().set(Symbol::short("swaps"), &swaps);
    }

    // View swap details
    pub fn view_swap(env: Env, swap_id: Symbol) -> Option<Swap> {
        let swaps = Self::swaps(&env);
        swaps.get(swap_id)
    }
}
