#!/bin/bash

contract=i.zxcvn.testnet

# near create-account $contract --masterAccount zxcvn.testnet --initialBalance 10
# near deploy $contract res/devgovgigs.wasm --initFunction new --initArgs '{}'

# for i in $(seq 1 2)
# do
# near call $contract add_post --accountId zxcvn.testnet --deposit 0.01 --args '{"parent_id":null,"body":{"post_type": "Idea","idea_version":"V1","name":"a'$i'","description":"aaa"},"labels":[]}'
# near call $contract add_post --accountId zxcvn.testnet --deposit 0.01 --args '{"parent_id":null,"body":{"post_type": "Idea","idea_version":"V1","name":"b'$i'","description":"bbb"},"labels":[]}'
# near call $contract add_post --accountId a.zxcvn.testnet --deposit 0.01 --args '{"parent_id":null,"body":{"post_type": "Idea","idea_version":"V1","name":"c'$i'","description":"ccc"},"labels":[]}'
# near call $contract add_post --accountId zxcvn.testnet --deposit 0.01 --args '{"parent_id":null,"body":{"post_type": "Idea","idea_version":"V1","name":"d'$i'","description":"ddd"},"labels":[]}'
# done

# near deploy $contract res/devgovgigs.wasm

# near call $contract unsafe_self_upgrade --accountId $contract --args $(base64 < res/devgovgigs.wasm ) --base64 --gas 300000000000000

#near call contract.devhubopen.testnet unsafe_self_upgrade --accountId contract.devhubopen.testnet --args $(base64 < res/devgovgigs.wasm ) --base64 --gas 300000000000000

# near call $contract unsafe_migrate --accountId $contract --gas 300000000000000

LIB_NEW="src/lib.rs"
STATE_VERSION_ENUM="src/migrations.rs"

# Extract the latest state version from migrations
latest_version=$(grep -o 'StateVersion::V[0-9]*' "$STATE_VERSION_ENUM" | tail -n 1)

# Extract the version mentioned in new
new_function_version=$(awk '/pub fn new/ {getline; print}' "$LIB_NEW" | grep -o 'StateVersion::V[0-9]*')

if [ "$latest_version" = "$new_function_version" ]; then
    exit 0
else
    echo "Test failed: Latest state version does not match the version in the new function."
    exit 1
fi