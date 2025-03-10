////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    farm_with_lock
    (
        callBack
        calculateRewardsForGivenPosition
        claimRewards
        compoundRewards
        end_produce_rewards
        enterFarm
        exitFarm
        getBurnGasLimit
        getDivisionSafetyConstant
        getFarmMigrationConfiguration
        getFarmTokenId
        getFarmTokenSupply
        getFarmingTokenId
        getLastErrorMessage
        getLastRewardBlockNonce
        getLockedAssetFactoryManagedAddress
        getMinimumFarmingEpoch
        getPairContractManagedAddress
        getPenaltyPercent
        getPerBlockRewardAmount
        getRewardPerShare
        getRewardReserve
        getRewardTokenId
        getState
        getTransferExecGasLimit
        mergeFarmTokens
        migrateFromV1_2Farm
        pause
        registerFarmToken
        resume
        setFarmMigrationConfig
        setFarmTokenSupply
        setLocalRolesFarmToken
        setPerBlockRewardAmount
        setRpsAndStartRewards
        set_burn_gas_limit
        set_minimum_farming_epochs
        set_penalty_percent
        set_transfer_exec_gas_limit
        startProduceRewards
    )
}
