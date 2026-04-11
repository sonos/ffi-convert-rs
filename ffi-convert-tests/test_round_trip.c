#include <stdio.h>
#include <string.h>
#include <assert.h>

#include "ffi_convert_tests.h"

void test_full_pancake(void) {
    CDummy dummy = {
        .count = 42,
        .describe = "hello",
    };

    CTopping toppings_data[] = {{.amount = 2}, {.amount = 3}};
    CArray_CTopping toppings = {
        .data_ptr = toppings_data,
        .size = 2,
    };

    CLayer layers_data[] = {
        {.number = 10, .subtitle = "inner"},
    };
    CArray_CLayer layers = {
        .data_ptr = layers_data,
        .size = 1,
    };

    CLayer base_layers[3] = {
        {.number = 0, .subtitle = "flour"},
        {.number = 1, .subtitle = "dough"},
        {.number = 2, .subtitle = "tomato"},
    };

    CSauce sauce = {.volume = 3.14f};
    float end = 5.0f;

    CRange_i32 range = {.start = 20, .end = 30};

    uint8_t data[] = {0x01, 0x02, 0x03};
    CArray_u8 pancake_data = {
        .data_ptr = data,
        .size = 3,
    };

    CPancake pancake = {
        .name = "Full pancake",
        .description = "A fully loaded pancake",
        .start = 1.0f,
        .end = &end,
        .float_array = {1.0f, 2.0f, 3.0f, 4.0f},
        .dummy = dummy,
        .sauce = &sauce,
        .toppings = &toppings,
        .layers = &layers,
        .base_layers = {base_layers[0], base_layers[1], base_layers[2]},
        .is_delicious = true,
        .range = range,
        .flattened_range_start = 10,
        .flattened_range_end = 20,
        .field_with_specific_c_name = "renamed",
        .pancake_data = &pancake_data,
        .extra_ice_cream_flavor = Strawberry,
    };

    const CPancake *result = pancake_round_trip(&pancake);
    assert(result != NULL);

    assert(strcmp(result->name, "Full pancake") == 0);
    assert(strcmp(result->description, "A fully loaded pancake") == 0);
    assert(result->start == 1.0f);
    assert(result->end != NULL);
    assert(*result->end == 5.0f);
    assert(result->float_array[0] == 1.0f);
    assert(result->float_array[3] == 4.0f);
    assert(result->dummy.count == 42);
    assert(strcmp(result->dummy.describe, "hello") == 0);
    assert(result->sauce != NULL);
    assert(result->sauce->volume == 3.14f);
    assert(result->toppings->size == 2);
    assert(result->toppings->data_ptr[0].amount == 2);
    assert(result->toppings->data_ptr[1].amount == 3);
    assert(result->layers != NULL);
    assert(result->layers->size == 1);
    assert(result->layers->data_ptr[0].number == 10);
    assert(strcmp(result->layers->data_ptr[0].subtitle, "inner") == 0);
    assert(result->base_layers[0].number == 0);
    assert(strcmp(result->base_layers[1].subtitle, "dough") == 0);
    assert(result->is_delicious == true);
    assert(result->range.start == 20);
    assert(result->range.end == 30);
    assert(result->flattened_range_start == 10);
    assert(result->flattened_range_end == 20);
    assert(strcmp(result->field_with_specific_c_name, "renamed") == 0);
    assert(result->pancake_data != NULL);
    assert(result->pancake_data->size == 3);
    assert(result->pancake_data->data_ptr[0] == 0x01);
    assert(result->pancake_data->data_ptr[2] == 0x03);
    assert(result->extra_ice_cream_flavor == Strawberry);

    pancake_free(result);
    printf("  full pancake: OK\n");
}

void test_minimal_pancake(void) {
    CDummy dummy = {
        .count = 0,
        .describe = "",
    };

    CArray_CTopping toppings = {
        .data_ptr = NULL,
        .size = 0,
    };

    CLayer base_layers[3] = {
        {.number = 0, .subtitle = NULL},
        {.number = 0, .subtitle = NULL},
        {.number = 0, .subtitle = NULL},
    };

    CRange_i32 range = {.start = 0, .end = 0};

    CPancake pancake = {
        .name = "",
        .description = NULL,
        .start = 0.0f,
        .end = NULL,
        .float_array = {0.0f, 0.0f, 0.0f, 0.0f},
        .dummy = dummy,
        .sauce = NULL,
        .toppings = &toppings,
        .layers = NULL,
        .base_layers = {base_layers[0], base_layers[1], base_layers[2]},
        .is_delicious = false,
        .range = range,
        .flattened_range_start = 0,
        .flattened_range_end = 0,
        .field_with_specific_c_name = "",
        .pancake_data = NULL,
        .extra_ice_cream_flavor = Vanilla,
    };

    const CPancake *result = pancake_round_trip(&pancake);
    assert(result != NULL);

    assert(strcmp(result->name, "") == 0);
    assert(result->description == NULL);
    assert(result->start == 0.0f);
    assert(result->end == NULL);
    assert(result->float_array[0] == 0.0f);
    assert(result->dummy.count == 0);
    assert(strcmp(result->dummy.describe, "") == 0);
    assert(result->sauce == NULL);
    assert(result->toppings->size == 0);
    assert(result->layers == NULL);
    assert(result->base_layers[0].number == 0);
    assert(result->base_layers[0].subtitle == NULL);
    assert(result->is_delicious == false);
    assert(result->range.start == 0);
    assert(result->range.end == 0);
    assert(result->flattened_range_start == 0);
    assert(result->flattened_range_end == 0);
    assert(strcmp(result->field_with_specific_c_name, "") == 0);
    assert(result->pancake_data == NULL);
    assert(result->extra_ice_cream_flavor == Vanilla);

    pancake_free(result);
    printf("  minimal pancake: OK\n");
}

void test_asan_canary(void) {
    /* Deliberately trigger a use-after-free to verify ASan is working. */
    CDummy dummy = {.count = 0, .describe = ""};
    CArray_CTopping toppings = {.data_ptr = NULL, .size = 0};
    CLayer base_layers[3] = {
        {.number = 0, .subtitle = NULL},
        {.number = 0, .subtitle = NULL},
        {.number = 0, .subtitle = NULL},
    };
    CRange_i32 range = {.start = 0, .end = 0};
    CPancake pancake = {
        .name = "canary",
        .description = NULL,
        .start = 0.0f,
        .end = NULL,
        .float_array = {0},
        .dummy = dummy,
        .sauce = NULL,
        .toppings = &toppings,
        .layers = NULL,
        .base_layers = {base_layers[0], base_layers[1], base_layers[2]},
        .is_delicious = false,
        .range = range,
        .flattened_range_start = 0,
        .flattened_range_end = 0,
        .field_with_specific_c_name = "",
        .pancake_data = NULL,
        .extra_ice_cream_flavor = Vanilla,
    };

    const CPancake *result = pancake_round_trip(&pancake);
    pancake_free(result);
    /* use-after-free: ASan should catch this */
    printf("  asan canary (use-after-free): name=%s\n", result->name);
}

void test_msan_canary(void) {
    /* Deliberately read uninitialized memory to verify MSan is working. */
    int uninit;
    /* volatile prevents the compiler from optimizing away the read */
    if (*(volatile int *)&uninit > 0) {
        printf("  msan canary: uninit was positive\n");
    } else {
        printf("  msan canary: uninit was non-positive\n");
    }
}

int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "--asan-canary") == 0) {
        printf("Triggering ASan canary (should crash):\n");
        test_asan_canary();
        printf("ERROR: ASan did not catch use-after-free!\n");
        return 1;
    }

    if (argc > 1 && strcmp(argv[1], "--msan-canary") == 0) {
        printf("Triggering MSan canary (should crash):\n");
        test_msan_canary();
        printf("ERROR: MSan did not catch uninitialized read!\n");
        return 1;
    }

    printf("C round-trip tests:\n");
    test_full_pancake();
    test_minimal_pancake();
    printf("All C round-trip tests passed!\n");
    return 0;
}
