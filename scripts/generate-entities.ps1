Write-Output "Generating entities..."
sea generate entity -o .\modules\srv-mod-entity\src\entities-new --with-serde both --ignore-tables kageshirei_migrations --serde-skip-deserializing-primary-key --serde-skip-hidden-column
Write-Output "Entities generated successfully."
Write-Output "You can now migrate and merge the updated entities from .\modules\srv-mod-entity\src\entities-new to .\modules\srv-mod-entity\src\entities"