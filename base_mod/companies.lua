data:extend {
    {
        type = "goods-company",
        order = "a-0",
        name = "bakery",
        label = "Bakery",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "store",
        recipe = {
            consumption = {{"flour", 1}},
            production = {{"bread", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 3,
        size = 10.0,
        asset_location = "bakery.glb",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "a-1",
        name = "flour-factory",
        label = "Flour Factory",
        bgen = {
            kind = "centered_door",
            vertical_factor = 0.6,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"cereal", 1}},
            production = {{"flour", 10}},
            complexity = 200,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "flour_factory.glb",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "a-2",
        name = "cereal-farm",
        label = "Cereal Farm",
        bgen = "farm",
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"cereal", 1}},
            complexity = 40,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 120.0,
        asset_location = "assets/sprites/dirt.jpg",
        price = 200,
        zone = {
            floor = "assets/sprites/dirt.jpg",
            filler = "wheat_up.glb",
            price_per_area = 100,
            randomize_filler = true,
        },
    },
    {
        type = "solar-panel",
        order = "b-1",
        name = "solar-panel",
        label = "Solar Panels",
        max_power = "1kW",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "network",
        recipe = {
            consumption = {},
            production = {},
            complexity = 1,
            storage_multiplier = 5,
        },
        n_workers = 0,
        size = 120.0,
        asset_location = "assets/sprites/cement.jpg",
        price = 0,
        zone = {
            floor = "assets/sprites/cement.jpg",
            filler = "solarpanel.glb",
            price_per_area = 10,
        },
    },
    {
        type = "goods-company",
        order = "b-2",
        name = "coal-power-plant",
        label = "Coal power plant",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "network",
        recipe = {
            consumption = {{"coal", 1}},
            production = {},
            complexity = 100,
            storage_multiplier = 5,
            power_generation = "2.46MW",
        },
        n_workers = 10,
        size = 165.0,
        asset_location = "coal_power_plant.glb",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "c-1",
        name = "supermarket",
        label = "Supermarket",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "store",
        recipe = {
            consumption = {{"meat", 1}, {"vegetable", 1}, {"cereal", 1}},
            production = {},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/supermarket.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "d-1",
        name = "clothes-store",
        label = "Clothes store",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "store",
        recipe = {
            consumption = {{"cloth", 1}},
            production = {},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 10.0,
        asset_location = "assets/sprites/clothes_store.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "d-2",
        name = "cloth-factory",
        label = "Cloth factory",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"polyester", 1}, {"wool", 1}},
            production = {{"cloth", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/cloth_factory.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "d-3",
        name = "textile-processing-facility",
        label = "Textile processing facility",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"wool", 1}},
            production = {{"cloth", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/textile_processing_facility.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "d-4",
        name = "polyester-refinery",
        label = "Polyester refinery",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"oil", 1}},
            production = {{"polyester", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 5,
        size = 80.0,
        asset_location = "assets/sprites/polyester_refinery.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "e-1",
        name = "oil-pump",
        label = "Oil pump",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"oil", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 5,
        size = 20.0,
        asset_location = "assets/sprites/oil_pump.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "e-2",
        name = "coal-mine",
        label = "Coal mine",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"coal", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 5,
        size = 20.0,
        asset_location = "assets/sprites/oil_pump.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "f-1",
        name = "wool-farm",
        label = "Wool farm",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"wool", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/wool_farm.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "g-1",
        name = "florist",
        label = "Florist",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "store",
        recipe = {
            consumption = {{"flower", 1}},
            production = {},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 10.0,
        asset_location = "assets/sprites/florist.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "g-2",
        name = "horticulturalist",
        label = "Horticulturalist",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"flower", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 5,
        size = 80.0,
        asset_location = "assets/sprites/horticulturalist.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "h-1",
        name = "high-tech-store",
        label = "High tech store",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "store",
        recipe = {
            consumption = {{"high-tech-product", 1}},
            production = {},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/hightech_store.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "h-2",
        name = "high-tech-facility",
        label = "High tech facility",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"gold", 1}, {"metal", 1}},
            production = {{"high-tech-product", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/hightech_facility.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "i-1",
        name = "iron-mine",
        label = "Iron mine",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"iron-ore", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/iron_mine.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "i-1",
        name = "gold-mine",
        label = "Gold mine",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"gold", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/rare_metal_mine.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "j-1",
        name = "lumber-yard",
        label = "Lumber yard",
        bgen = "farm",
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"tree-log", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 200.0,
        asset_location = "assets/sprites/lumber_yard.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "j-2",
        name = "woodmill",
        label = "Woodmill",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"tree-log", 1}},
            production = {{"wood-plank", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/woodmill.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "j-3",
        name = "furniture-store",
        label = "Furniture store",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "store",
        recipe = {
            consumption = {{"metal", 1}, {"wood-plank", 1}},
            production = {{"furniture", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/furniture_store.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "h-1",
        name = "foundry",
        label = "Foundry",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"iron-ore", 1}},
            production = {{"metal", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/foundry.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "k-1",
        name = "slaughterhouse",
        label = "Slaughterhouse",
        bgen = {
            kind = "centered_door",
            vertical_factor = 1.0,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"carcass", 1}},
            production = {{"raw-meat", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 5,
        size = 50.0,
        asset_location = "assets/sprites/slaughterhouse.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "k-1",
        name = "animal-farm",
        label = "Animal Farm",
        bgen = "farm",
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"cereal", 1}},
            production = {{"carcass", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 5,
        size = 80.0,
        asset_location = "assets/sprites/animal_farm.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "k-2",
        name = "meat-facility",
        label = "Meat facility",
        bgen = {
            kind = "centered_door",
            vertical_factor = 0.6,
        },
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {{"raw-meat", 1}},
            production = {{"meat", 1}},
            complexity = 100,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 80.0,
        asset_location = "assets/sprites/meat_facility.png",
        price = 1000,
    },
    {
        type = "goods-company",
        order = "l-1",
        name = "vegetable-farm",
        label = "Vegetable Farm",
        bgen = "farm",
        kind = "factory",
        n_trucks = 1,
        recipe = {
            consumption = {},
            production = {{"vegetable", 2}},
            complexity = 2,
            storage_multiplier = 5,
        },
        n_workers = 10,
        size = 70.0,
        asset_location = "assets/sprites/vegetable_farm.png",
        price = 1000,
        zone = {
            floor = "assets/sprites/dirt.jpg",
            filler = "salad.glb",
            price_per_area = 100,
        },
    },
}
