Below is the markup for the README section, including the explanations and code snippets you provided. This markup is written in Markdown, which is commonly used for README files on GitHub. Copy and paste this into your README.md file for it to display correctly on your GitHub repository.

```markdown
## Functionality

This application includes functionality for locking and unlocking assets within a smart contract on the Terra blockchain. Below are the details on how to use these features in the code.

### Locking Assets

To lock assets, the application constructs a message to execute a smart contract on the Terra blockchain. This is done by specifying the amount to lock and a memo for the transaction. Here's an example of how to lock `1,000,000` units of an asset:

```javascript
case 'lock':
    const amountToLock = '1000000'; // Specify the amount to lock
    const lockMsg = {
        lock: {
            memo: 'I got $BEEF yo!' // Add a memo to the lock transaction
        }
    };
    msg = new MsgExecuteContract(
        wallet.key.accAddress, // Your Terra wallet address
        'terra164kf48vusvnmsku8v37uy9ynxpr5u333hvcz0wd6mfr8el56wx9sfzuhxq', // The address of the contract to execute
        {
            send: {
                contract: 'terra126krz76md6ykyvnltme2u5uhl640r7pcnsec9kcmnaak9vm7agjqcq3y0q', // The contract to which assets are being locked
                amount: amountToLock,
                msg: Buffer.from(JSON.stringify(lockMsg)).toString('base64') // The lock message, base64 encoded
            }
        },
        {} // Additional options can be specified here
    );
    break;
```

### Unlocking Assets

To unlock assets, you need to specify the `lock_id` that was generated when the assets were locked. This example demonstrates how to unlock assets:

```javascript
case 'unlock':
    const lockIdToUnlock = 'terra10p0n98g39xj8k5twpd99c068s6ykuzgcv28a5k_terra164kf48vusvnmsku8v37uy9ynxpr5u333hvcz0wd6mfr8el56wx9sfzuhxq_1'; // The lock_id of the assets to unlock
    msg = new MsgExecuteContract(
        wallet.key.accAddress, // Your Terra wallet address
        'terra126krz76md6ykyvnltme2u5uhl640r7pcnsec9kcmnaak9vm7agjqcq3y0q', // The address of the contract to execute
        {
            unlock: {
                lock_id: lockIdToUnlock // Specify the lock_id to unlock
            },
        },
        {} // Additional options can be specified here
    );
    break;
```

```
