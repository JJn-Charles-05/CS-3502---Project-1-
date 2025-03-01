use std::sync::{Arc,Mutex}; //Rust's library of thread-safe tools. Allows threads to share ownership
// of a value
use std::sync::atomic::{AtomicUsize, Ordering}; //A thread-safe auto incrementing ID generating tool
use std::thread;
use std::time::Duration;

// User bank account structure; stores user-specific banking data
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
struct BankAccount {
    account_name: String,
    account_id: usize,
    account_balance: f64,
}

// "Constructor"/implementation of BankAccount structure
impl BankAccount {
    fn new(name: &str, init_bal: f64,) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        BankAccount{
            account_name: name.to_string(),
            account_id: id,
            account_balance: init_bal,
        }
    }
    fn get_account_info(&self) { // Prints out general account info
        println!("--------------------------");
        println!("Welcome, {}! Account Info:", self.account_name);
        println!("Account ID: {}", self.account_id);
        println!("Account Balance: {}", self.account_balance);
        println!("--------------------------");
    }
    fn account_bal(&self) -> f64 { // Outputs the account's currents balance
        self.account_balance
    }
    fn deposit(&mut self, amnt: f64) { // Permits account holder to deposit money
        self.account_balance += amnt;
    }
    fn withdraw(&mut self, amnt: f64) { // Permits account holder to withdraw money if they have enough
        if self.account_balance >= amnt {
            self.account_balance -= amnt;
        } else {
            println!("The account for {} has insufficient funds to make this transaction.", self.account_name);
        }
    }
    // This function has the potential to cause a deadlock scenario.
    // Implementing this feature with ordered account access avoids deadlocks entirely.
    // However, deadlock detection has been implemented anyway.
    pub fn transfer (sender: &Arc<Mutex<BankAccount>>, receiver: &Arc<Mutex<BankAccount>>, amount: f64){
        //Enforce order of threads based on order in memory.
        let (t_first, t_second) = if Arc::as_ptr(sender) < Arc::as_ptr(receiver) {
            (sender, receiver)
        } else {
            (receiver, sender)
        };

        let acc_first_lock = t_first.try_lock();
        if acc_first_lock.is_err() {
            println!("!- Warning -! Potential deadlock scenario detected! Locking on first account unavailable!")
        }
        let mut acc_first = acc_first_lock.unwrap(); //Lock thread that's first in memory
        thread::sleep(Duration::from_millis(100)); //Simulate delay

        let acc_second_lock = t_second.try_lock();
        if acc_second_lock.is_err() {
            println!("!- Warning -! Potential deadlock scenario detected! Locking on second account unavailable!")
        }
        let mut acc_second = acc_second_lock.unwrap(); //Lock thread that's second in memory

        if acc_first.account_balance >= amount {
            acc_first.withdraw(amount);
            acc_second.deposit(amount); //If the sender has sufficient funds, transfer.
            println!("${} successfully transferred from {} to {}!", amount, acc_first.account_name, acc_second.account_name);
        } else {
            println!("The account for {} has insufficient funds to make this transaction.", acc_first.account_name); //Else, print error message.
        }
    }
}

fn main() {
    // Generate a thread-safe, shared account via Arc and Mutex
    let account = Arc::new(Mutex::new(BankAccount::new("Account Holder", 10000.0)));
    let mut handles = vec![]; // Storing thread handles produced; mutable

    println!("!! - Demonstrating thread creation and synchronization through mutexes. ~~\n");
    for _ in 0..10 { //Spawn and manipulate 5 threads
        let account_clone = Arc::clone(&account); // Create a clone of acc with Arc to share ownership
        let handle = thread::spawn(move || { //Create new thread
            let mut acc = account_clone.lock().unwrap(); //Mutex lock for safe data access
            acc.deposit(100.0); // dep. 100 into account
            acc.withdraw(50.0); //with. 50 from account
            println!("Updated {} Account Balance: {}", acc.account_name, acc.account_bal()); //Print updated balance
        });
        handles.push(handle); //Store current thread handle

        for handle in handles.drain(..) {
            handle.join().unwrap();
        }
    }
    println!("\n[ ! ] Your account has been updated! [ ! ]\n");
    account.lock().unwrap().get_account_info();

    println!("\n!! - Demonstrating deadlock management through transfer protocol. ~~\n");
    let sender_1 = Arc::new(Mutex::new(BankAccount::new("Sender1 Account", 500000.0)));
    let receiver_1 = Arc::new(Mutex::new(BankAccount::new("Receiver1 Account", 2000.0)));
    BankAccount::transfer(&sender_1, &receiver_1, 3000.0);

    println!("\n! - An example that produces an error code:\n");
    let sender_2 = Arc::new(Mutex::new(BankAccount::new("Sender2 Account", 100.0)));
    let receiver_2 = Arc::new(Mutex::new(BankAccount::new("Receiver2 Account", 10000.0)));
    BankAccount::transfer(&sender_2, &receiver_2, 3000.0);
}

//Thread & function testing for the solution.
mod tests {
    use super::*; // Imports all from the file so that it is in this scope

    #[test]
    fn deposit_test() { // Verify deposit()'s functionality
        let mut test_acc = BankAccount::new("Test Account", 1000.0);
        test_acc.deposit(500.0);
        assert_eq!(test_acc.account_bal(), 1500.0, "Deposit function FAIL!");
    }

    #[test]
    fn withdraw_success_test() { // Verify withdraw()'s functionality
        let mut test_acc = BankAccount::new("Test Account", 1000.0);
        test_acc.withdraw(500.0);
        assert_eq!(test_acc.account_bal(), 500.0, "Withdraw function (success case) FAIL!");
    }

    #[test]
    fn withdraw_fail_test() { // Verify withdraw()'s functionality (Fail case)
        let mut test_acc = BankAccount::new("Test Account", 500.0);
        test_acc.withdraw(1000.0);
        assert_eq!(test_acc.account_bal(), 500.0, "Withdraw function (fail case) FAIL! (Should not reduce the balance)");
    }

    #[test]
    fn transfer_success_test() { // Verify transfer()'s functionality
        let mut test_send_acc = Arc::new(Mutex::new(BankAccount::new("Send Account", 1000.0)));
        let mut test_receive_acc = Arc::new(Mutex::new(BankAccount::new("Receive Account", 100.0)));

        BankAccount::transfer(&test_send_acc, &test_receive_acc, 400.0);

        let test_send_acc_bal = test_send_acc.lock().unwrap().account_bal();
        let test_receive_acc_bal = test_receive_acc.lock().unwrap().account_bal();

        assert_eq!(test_send_acc_bal, 600.0, "Transfer function (success case) FAIL! (Sender acc bal incorrect)");
        assert_eq!(test_receive_acc_bal, 500.0, "Transfer function (success case) FAIL! (Receiver acc bal incorrect)");
    }

    #[test]
    fn transfer_fail_test() { // Verify transfer()'s functionality (Fail case)
        let mut test_send_acc = Arc::new(Mutex::new(BankAccount::new("Send Account", 200.0)));
        let mut test_receive_acc = Arc::new(Mutex::new(BankAccount::new("Receive Account", 1000.0)));

        BankAccount::transfer(&test_send_acc, &test_receive_acc, 400.0);

        let test_send_acc_bal = test_send_acc.lock().unwrap().account_bal();
        let test_receive_acc_bal = test_receive_acc.lock().unwrap().account_bal();

        assert_eq!(test_send_acc_bal, 200.0, "Transfer function (fail case) FAIL! (Sender acc bal incorrect)");
        assert_eq!(test_receive_acc_bal, 1000.0, "Transfer function (fail case) FAIL! (Receiver acc bal incorrect)");
    }

    #[test]
    //Test doubles as sync. validation
    fn high_load_stress_test() { //Verify program capability in high-stress environment
        let account = Arc::new(Mutex::new(BankAccount::new("Account Holder", 10000.0)));
        let mut handles = vec![]; // Storing thread handles produced; mutable
        for _ in 0..1000 { //Spawn and manipulate 1000 threads
            let account_clone = Arc::clone(&account); // Create a clone of acc with Arc to share ownership
            let handle = thread::spawn(move || { //Create new thread
                let mut acc = account_clone.lock().unwrap(); //Mutex lock for safe data access
                acc.deposit(100.0); // dep. 100 into account
                acc.withdraw(50.0); //with. 50 from account
                println!("Updated {} Account Balance: {}", acc.account_name, acc.account_balance); //Print updated balance
            });
            handles.push(handle); //Store current thread handle

            for handle in handles.drain(..) {
                handle.join().unwrap();
            }
        }
        let final_bal_check = account.lock().unwrap().account_bal();
        assert_eq!(final_bal_check, 60000.0, "Stress test failed: Final bal. INCORRECT!");
    }
}