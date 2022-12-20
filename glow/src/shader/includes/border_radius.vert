float selectBorderRadius(vec4 radi, vec2 position, vec2 center)
{
    float rx = radi.x;
    float ry = radi.y;
    rx = position.x > center.x ? radi.y : radi.x;
    ry = position.x > center.x ? radi.z : radi.w;
    rx = position.y > center.y ? ry : rx;
    return rx;
}
