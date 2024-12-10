#version 460

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;



struct Cell {
    float terrain_height;
    float water_height;
    float suspended_sediment;
    float sediment_capacity;
    vec4 outflow_flux;
    vec2 velocity;
};



/// BUFFERS


layout(set = 0, binding = 0, rgba8) uniform image2D rain_map;

layout(set = 0, binding = 1) buffer Cells {
    Cell[] cells;
};



layout(push_constant) uniform PushConstants {
    // int num_cells;
    float delta_time;
    float rainfall_scale;

    float pipe_area;
    float gravity;
    float pipe_length;
    float cell_width;
    float cell_height;

    float sediment_capacity_scaler;
    float max_erosion_depth;

    float dissolving_constant;
    float deposition_constant;

    uint width;
    uint height;
    uint sim_step;
} push_constants;





/// get the differences in total heights between a cell and its 4 neighbours
vec4 get_height_diffs(uint x, uint y) {
    vec4 diffs = vec4(0);
    uint id = x + y * push_constants.width;
    float cell_height = cells[id].terrain_height + cells[id].water_height;
    
    if (x != 0) {
        diffs.x = cell_height - cells[id - 1].terrain_height - cells[id - 1].water_height;
    }

    if (x != push_constants.width - 1) {
        diffs.y = cell_height - cells[id + 1].terrain_height - cells[id + 1].water_height;
    }

    if (y != 0) {
        diffs.z = cell_height - cells[id + push_constants.width].terrain_height - cells[id + push_constants.width].water_height;
    }

    if (y != push_constants.height - 1) {
        diffs.z = cell_height - cells[id - push_constants.width].terrain_height - cells[id - push_constants.width].water_height;
    }


    return diffs;
}


vec4 get_incoming_water(uint x, uint y) {
    vec4 incoming = vec4(0);
    uint id = x + y * push_constants.width;
    
    if (x != 0) {
        incoming.x = cells[id - 1].outflow_flux.y;
    }
    if (x != push_constants.width - 1) {
        incoming.y = cells[id + 1].outflow_flux.x;
    }
    if (y != 0) {
        incoming.z = cells[id - push_constants.width].outflow_flux.w;   
    }
    if (y != push_constants.height - 1) {
        incoming.w = cells[id + push_constants.width].outflow_flux.z;  
    }

    return incoming;
}


float vec4_sum(vec4 x) {
    return x.x + x.y + x.z + x.w;
}


float l_max(float x) {
    if (x <= 0) {
        return 0.0;
    }
    else if (x >= push_constants.max_erosion_depth) {
        return 1.0;
    } 
    else {
        return 1 - (push_constants.max_erosion_depth - x);
    }
}



float sin_of_local_tilt_angle(uint x, uint y) {

    float dh_dx;
    float dh_dy;

    uint id = x + y * push_constants.width;
    float cell_height = cells[id].terrain_height;
    
    if (x == 0) {
        dh_dx = cells[id + 1].terrain_height / push_constants.cell_width;
    } else if (x == push_constants.width - 1) {
        dh_dx = -cells[id - 1].terrain_height / push_constants.cell_width;
    } else {
        dh_dx = (cells[id + 1].terrain_height - cells[id - 1].terrain_height) / (2.0 * push_constants.cell_width);
    }

    if (y == 0) {
        dh_dy = cells[id - push_constants.width].terrain_height / push_constants.cell_height;
    } else if (y == push_constants.width - 1) {
        dh_dy = -cells[id - push_constants.width].terrain_height / push_constants.cell_height;
    } else {
        dh_dy = (cells[id - push_constants.width].terrain_height - cells[id - push_constants.width].terrain_height) / (2.0 * push_constants.cell_height);
    }

    
    return sqrt(pow(dh_dx, 2) + pow(dh_dx, 2)) / sqrt(1.0 + pow(dh_dx, 2) + pow(dh_dx, 2));
}


void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    if (x > push_constants.width || y > push_constants.height) {
        return;
    }

    uint id = x + y * push_constants.width;


    // steps

    Cell cell = cells[id];

    // 1 - Water Incrementation
    if (push_constants.sim_step == 0) {
        cell.water_height += push_constants.delta_time * push_constants.rainfall_scale * imageLoad(rain_map, ivec2(x, y)).x;

        return;
    }

    // 2 - Update Outflow Flux
    if (push_constants.sim_step == 1) {
        vec4 new_outflow_flux = max(vec4(0), cell.outflow_flux + push_constants.delta_time * push_constants.pipe_area / push_constants.pipe_length * get_height_diffs(x, y));
        float k = max(1, (cell.water_height * push_constants.cell_width * push_constants.cell_height) / (vec4_sum(new_outflow_flux) * push_constants.delta_time));
        cell.outflow_flux = new_outflow_flux * k;

        return;
    }


    // 3 - Calculate water height change
    if (push_constants.sim_step == 2) {
        vec4 incoming = get_incoming_water(x, y);
        float delta_water = push_constants.delta_time * (vec4_sum(incoming) - vec4_sum(cell.outflow_flux));


        // calculate velocity field
        float delta_vel_x = 0.5 * (incoming.x - cell.outflow_flux.x + cell.outflow_flux.y - incoming.y);
        float delta_vel_y = 0.5 * (incoming.z - cell.outflow_flux.z + cell.outflow_flux.w - incoming.w);
        cell.velocity += vec2(delta_vel_x, delta_vel_y);


        // calculate sediment capacity
        cell.sediment_capacity = push_constants.sediment_capacity_scaler * sin_of_local_tilt_angle(x, y) * length(cell.velocity);

        // add the water
        cell.water_height += delta_water / (push_constants.cell_width * push_constants.cell_height);

        return;
    }


    // 4 - Erosion-Deposition
    if (push_constants.sim_step == 3) {
        
        // erode
        // if (cell.sediment_capacity > cell.suspended_sediment) {
            
        // }


        // deposit

    }

    // 5 - Transport of Suspeneded_sediment

    // 6 - Water Evaporation


}


