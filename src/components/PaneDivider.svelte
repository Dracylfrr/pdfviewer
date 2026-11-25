<script lang="ts">
  let {label, value, min, max, reverse=false, resize}: {
    label:string; value:number; min:number; max:number; reverse?:boolean; resize:(width:number)=>void
  }=$props();
  let drag:{id:number;x:number;width:number}|null=null;
  function setWidth(width:number){resize(Math.max(min,Math.min(max,Math.round(width))))}
  function pointerDown(event:PointerEvent){
    if(event.button!==0)return;
    const handle=event.currentTarget as HTMLElement;
    handle.focus({preventScroll:true});
    handle.setPointerCapture(event.pointerId);
    drag={id:event.pointerId,x:event.clientX,width:value};
  }
  function pointerMove(event:PointerEvent){
    if(drag?.id===event.pointerId)setWidth(drag.width+(event.clientX-drag.x)*(reverse?-1:1));
  }
  function keydown(event:KeyboardEvent){
    const step=event.shiftKey?48:12;
    if(event.key==='Home')setWidth(min);
    else if(event.key==='End')setWidth(max);
    else if(event.key==='ArrowLeft'||event.key==='ArrowRight'){
      setWidth(value+step*(event.key==='ArrowRight'?1:-1)*(reverse?-1:1));
    }else return;
    event.preventDefault();
  }
</script>
<!-- A window splitter is a focusable separator with a value and arrow-key controls. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div class="pane-divider" role="separator" tabindex="0" aria-label={label}
  aria-orientation="vertical" aria-valuemin={min} aria-valuemax={max} aria-valuenow={value}
  title={label} onpointerdown={pointerDown} onpointermove={pointerMove}
  onlostpointercapture={()=>drag=null} onkeydown={keydown}></div>
