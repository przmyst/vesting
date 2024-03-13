  case 'lock':
      const amountToLock = '1000000';
      const lockMsg = {
          lock: {
              memo: 'I got $BEEF yo!'
          }
      };
      msg = new MsgExecuteContract(
          wallet.key.accAddress,
         'terra164kf48vusvnmsku8v37uy9ynxpr5u333hvcz0wd6mfr8el56wx9sfzuhxq',
          {
              send: {
                  contract: 'terra126krz76md6ykyvnltme2u5uhl640r7pcnsec9kcmnaak9vm7agjqcq3y0q',
                  amount: amountToLock,
                  msg: Buffer.from(JSON.stringify(lockMsg)).toString('base64')
              }
          },
          {}
      );
      break;

  case 'unlock':
      const lockIdToUnlock = 'terra10p0n98g39xj8k5twpd99c068s6ykuzgcv28a5k_terra164kf48vusvnmsku8v37uy9ynxpr5u333hvcz0wd6mfr8el56wx9sfzuhxq_1'; // Assuming the lock_id is passed as the third command line argument
      msg = new MsgExecuteContract(
          wallet.key.accAddress,
          'terra126krz76md6ykyvnltme2u5uhl640r7pcnsec9kcmnaak9vm7agjqcq3y0q',
          {
              unlock: {
                  lock_id: lockIdToUnlock
              },
          },
          {}
      );
      break;
